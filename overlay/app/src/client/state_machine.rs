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
trait SlimeSinkT: Sink<Data, Error = WsError> + Send + Sync + Debug {}
impl<T> SlimeSinkT for T where T: Sink<Data, Error = WsError> + Send + Sync + Debug {}

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
	pub(super) fn into_state<Next>(self, state: Next) -> ClientStateMachine<Next> {
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
	pub async fn request_feed(mut self) -> Result<M<Active>, RecvError> {
		use solarxr_protocol::MessageBundleArgs;
		let fbb = &mut self.state.fbb;
		let data = {
			let data_feed_header = {
				use solarxr_protocol::data_feed::tracker::{
					TrackerDataMask, TrackerDataMaskArgs,
				};
				use solarxr_protocol::data_feed::{
					DataFeedConfig, DataFeedConfigArgs, DataFeedMessage,
					DataFeedMessageHeader, DataFeedMessageHeaderArgs, StartDataFeed,
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
				header
			};
			let pub_sub_header = {
				use crate::client::topic::{
					TOPIC_APP, TOPIC_DISPLAY_SETTINGS, TOPIC_ORG,
				};
				use solarxr_protocol::pub_sub::PubSubUnion;
				use solarxr_protocol::pub_sub::{PubSubHeader, PubSubHeaderArgs};
				use solarxr_protocol::pub_sub::{
					SubscriptionRequest, SubscriptionRequestArgs, Topic, TopicId,
					TopicIdArgs,
				};

				let organization = fbb.create_string(TOPIC_ORG);
				let app_name = fbb.create_string(TOPIC_APP);
				let topic = fbb.create_string(TOPIC_DISPLAY_SETTINGS);
				let topic = TopicId::create(
					fbb,
					&TopicIdArgs {
						organization: Some(organization),
						app_name: Some(app_name),
						topic: Some(topic),
						..Default::default()
					},
				);

				let subscription_request = SubscriptionRequest::create(
					fbb,
					&SubscriptionRequestArgs {
						topic_type: Topic::TopicId,
						topic: Some(topic.as_union_value()),
						..Default::default()
					},
				);

				let header = PubSubHeader::create(
					fbb,
					&PubSubHeaderArgs {
						u_type: PubSubUnion::SubscriptionRequest,
						u: Some(subscription_request.as_union_value()),
						..Default::default()
					},
				);
				let header = fbb.create_vector(&[header]);
				header
			};
			let root = MessageBundle::create(
				fbb,
				&MessageBundleArgs {
					data_feed_msgs: Some(data_feed_header),
					pub_sub_msgs: Some(pub_sub_header),
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
		if let Err(err) = sink.send(data).await {
			return Err(RecvError::CriticalWs(self.into_state(Disconnected), err));
		}

		// TODO: Actually get the TopicMapping

		// Wait until we get a `TopicMapping` in response to our `SubscriptionRequest`
		// let first_attempt = std::time::Instant::now();
		// loop {
		// 	if first_attempt.elapsed() >= Duration::from_millis(1000) {
		// 		return Err(RecvError::NoTopicMapping(self.into_state(Disconnected)));
		// 	}
		//
		// 	use RecvError as E;
		// 	match self.state.stream.next().await {
		// 		Some(Ok(v)) => {
		// 			todo!()
		// 		}
		// 		Some(Err(DeserializeError::Ws(ws_err))) => {
		// 			return Err(E::CriticalWs(self.into_state(Disconnected), ws_err))
		// 		}
		// 		None => return Err(RecvError::None(self.into_state(Disconnected))),
		// 	}
		// }

		Ok(M {
			common: self.common,
			state: Active {
				_sink: self.state.sink,
				stream: self.state.stream,
				topic_handle: 0,
			},
		})
	}
}

/// Datafeed is active
#[derive(Debug)]
pub struct Active {
	_sink: Pin<SlimeSink>,
	stream: SlimeStream,
	topic_handle: u32,
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
	#[error("No `TopicMapping` in response to `SubscriptionRequest`")]
	NoTopicMapping(M<Disconnected>),
}

pub type RecvResult = Result<(M<Active>, FeedUpdate), RecvError>;
