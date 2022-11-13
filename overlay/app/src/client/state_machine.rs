use super::data::FeedUpdate;
use super::Wss;
use crate::client::{Data, DecodeError};

use futures_util::stream::SplitStream;
use futures_util::{Sink, SinkExt, StreamExt};
use solarxr_protocol::flatbuffers::FlatBufferBuilder;
use solarxr_protocol::MessageBundle;
use std::fmt::Debug;
use std::future;
use std::pin::Pin;
use tokio_tungstenite::{connect_async, tungstenite};
use tungstenite::error::Error as WsError;
use tungstenite::Message;

type DeserializeFn = fn(Result<Message, WsError>) -> Result<Data, DeserializeError>;
type SlimeStream = futures_util::stream::Map<SplitStream<Wss>, DeserializeFn>;

// The need for this trait object is cringe
type SlimeSink = Box<dyn SlimeSinkT>;
trait SlimeSinkT: Sink<Data, Error = WsError> + Send + Debug {}
impl<T> SlimeSinkT for T where T: Sink<Data, Error = WsError> + Send + Debug {}

/// Data common to all states goes here
#[derive(Debug)]
struct Common {
	connect_to: String,
}
#[derive(Debug)]
pub struct ClientStateMachine<State = Disconnected> {
	state: State,
	common: Common,
}
impl ClientStateMachine {
	/// Creates a new `NetworkStateMachine`. This starts in the [`Disconnected`] state.
	pub fn new(connect_to: String) -> Self {
		Self {
			state: Disconnected,
			common: Common { connect_to },
		}
	}
}
impl<S> ClientStateMachine<S> {
	/// Helper function to transition to next state while preserving all common data
	fn into_state<Next>(self, state: Next) -> ClientStateMachine<Next> {
		ClientStateMachine {
			common: self.common,
			state,
		}
	}
}

// Makes things easier to type
type M<S> = ClientStateMachine<S>;

// ---- The different states of the state machine ----

#[derive(thiserror::Error, Debug)]
pub enum DeserializeError {
	#[error("Decoding error: {1}")]
	DecodeError(Vec<u8>, #[source] DecodeError),
	#[error("Payload was an invalid type")]
	PayloadType(Message),
	#[error(transparent)]
	Ws(#[from] WsError),
}

/// Client is fully disconnected from the server.
#[derive(Debug)]
pub struct Disconnected;
impl M<Disconnected> {
	pub async fn connect(self) -> Result<M<Connected>, (Self, WsError)> {
		match connect_async(&self.common.connect_to).await {
			Ok((socket, _)) => {
				let (sink, stream) = socket.split();

				// We never actually error, but this signature satisfies `sink.with()`
				// `Ready` is required because otherwise our `Future` won't implement `Debug`
				fn serialize(data: Data) -> future::Ready<Result<Message, WsError>> {
					let v = data.into_vec();
					log::trace!("Sending serialized data: {v:?}");
					future::ready(Ok(Message::Binary(v)))
				}

				fn deserialize(
					msg: Result<Message, WsError>,
				) -> Result<Data, DeserializeError> {
					log::trace!("Received websocket message");
					use DeserializeError::*;
					match msg {
						Ok(Message::Binary(v)) => {
							Data::from_vec(v).map_err(|e| DecodeError(e.0, e.1))
						}
						Ok(m) => Err(PayloadType(m)),
						Err(err) => Err(Ws(err)),
					}
				}

				let sink = sink.with(serialize);
				let sink: Pin<Box<dyn SlimeSinkT>> = Box::pin(sink);
				let stream: SlimeStream = stream.map(deserialize);
				Ok(self.into_state(Connected {
					sink,
					stream,
					fbb: FlatBufferBuilder::new(),
				}))
			}
			Err(e) => Err((self.into_state(Disconnected), e)),
		}
	}
}

/// Client is connected over websocket
#[derive(Debug)]
pub struct Connected {
	sink: Pin<SlimeSink>,
	stream: SlimeStream,
	fbb: FlatBufferBuilder<'static>,
}
impl M<Connected> {
	pub async fn request_feed(
		mut self,
	) -> Result<M<Active>, (M<Disconnected>, WsError)> {
		use solarxr_protocol::{
			data_feed::{DataFeedMessageHeader, DataFeedMessageHeaderArgs},
			MessageBundleArgs,
		};
		let fbb = &mut self.state.fbb;
		let data = {
			use solarxr_protocol::data_feed::tracker::{
				TrackerDataMask, TrackerDataMaskArgs,
			};
			use solarxr_protocol::data_feed::{
				DataFeedConfig, DataFeedConfigArgs, DataFeedMessage, StartDataFeed,
				StartDataFeedArgs,
			};

			let _tracker_mask = TrackerDataMask::create(
				fbb,
				&TrackerDataMaskArgs {
					// TODO: We only need the body part here, not the whole TrackerInfo
					info: true,
					rotation: true,
					position: true,
					..Default::default()
				},
			);

			let data_feed_config = DataFeedConfig::create(
				fbb,
				&DataFeedConfigArgs {
					minimum_time_since_last: 10,
					// We don't care about anything but bones
					bone_mask: true,
					..Default::default()
				},
			);
			let data_feed_config = fbb.create_vector(&[data_feed_config]);

			let start_data_feed = StartDataFeed::create(
				fbb,
				&StartDataFeedArgs {
					data_feeds: Some(data_feed_config),
				},
			);
			let header = DataFeedMessageHeader::create(
				fbb,
				&DataFeedMessageHeaderArgs {
					message_type: DataFeedMessage::StartDataFeed,
					message: Some(start_data_feed.as_union_value()),
					..Default::default()
				},
			);
			let header = fbb.create_vector(&[header]);
			let root = MessageBundle::create(
				fbb,
				&MessageBundleArgs {
					data_feed_msgs: Some(header),
					..Default::default()
				},
			);
			fbb.finish(root, None);
			let v = fbb.finished_data().to_vec();

			#[cfg(not(debug_assertions))]
			unsafe {
				Data::from_vec_unchecked(v)
			}
			#[cfg(debug_assertions)]
			Data::from_vec(v).unwrap()
		};

		let mut sink = self.state.sink.as_mut();
		match sink.send(data).await {
			Ok(()) => Ok(M {
				common: self.common,
				state: Active {
					_sink: self.state.sink,
					stream: self.state.stream,
				},
			}),
			Err(err) => Err((
				M {
					common: self.common,
					state: Disconnected,
				},
				err,
			)),
		}
	}
}

/// Datafeed is active
#[derive(Debug)]
pub struct Active {
	_sink: Pin<SlimeSink>,
	stream: SlimeStream,
}
impl M<Active> {
	pub async fn recv(mut self) -> RecvResult {
		use RecvError as E;
		match self.state.stream.next().await {
			Some(Ok(v)) => Ok((self, FeedUpdate(v))),
			Some(Err(DeserializeError::Ws(ws_err))) => {
				Err(E::CriticalWs(self.into_state(Disconnected), ws_err))
			}
			Some(Err(err)) => Err(E::Deserialize(self, err)),
			None => Err(E::None(self.into_state(Disconnected))),
		}
	}
}

#[derive(thiserror::Error, Debug)]
pub enum RecvError {
	#[error("Critical websocket error: {1}")]
	CriticalWs(M<Disconnected>, WsError),
	#[error("Error while deserializing: {1}")]
	Deserialize(M<Active>, DeserializeError),
	#[error("Stream produced `None`")]
	None(M<Disconnected>),
}

pub type RecvResult = Result<(M<Active>, FeedUpdate), RecvError>;
