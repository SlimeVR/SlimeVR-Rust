use super::data::FeedUpdate;
use super::Wss;
use crate::client::{Data, DataResult};

use eyre::{eyre, Result, WrapErr};
use futures_util::stream::SplitStream;
use futures_util::{Sink, SinkExt, StreamExt};
use solarxr_protocol::flatbuffers::FlatBufferBuilder;
use solarxr_protocol::MessageBundle;
use std::pin::Pin;
use tokio_tungstenite::{connect_async, tungstenite};
use tungstenite::error::Error as WsError;
use tungstenite::Message;

type DeserializeFn = fn(Result<Message, WsError>) -> Result<Data>;
type SlimeStream = futures_util::stream::Map<SplitStream<Wss>, DeserializeFn>;
type SlimeSink = Box<dyn Sink<Data, Error = WsError> + Send>; // Cringe

/// Data common to all states goes here
struct Common {
    connect_to: String,
}
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

type M<S> = ClientStateMachine<S>;

// ---- The different states of the state machine ----

/// Client is fully disconnected from the server.
pub struct Disconnected;
impl M<Disconnected> {
    pub async fn connect(self) -> Result<M<Connected>, (Self, eyre::Report)> {
        match connect_async(&self.common.connect_to)
            .await
            .wrap_err("Could not open websocket connection")
        {
            Ok((socket, _)) => {
                let (sink, stream) = socket.split();
                async fn serialize(data: Data) -> Result<Message, WsError> {
                    let v = data.into_vec();
                    log::trace!("Sending serialized data: {v:?}");
                    Ok(Message::Binary(v))
                }
                fn deserialize(msg: Result<Message, WsError>) -> Result<Data> {
                    log::trace!("Received message: {msg:?}");
                    match msg {
                        Ok(Message::Binary(v)) => {
                            Ok(Data::from_vec(v).wrap_err("Invalid message")?)
                        }
                        Ok(m) => Err(eyre!("Invalid websocket payload type: {m:?}")),
                        Err(err) => Err(err).wrap_err("Encountered error while deserializing"),
                    }
                }

                let sink = Box::pin(sink.with(serialize));
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
pub struct Connected {
    sink: Pin<SlimeSink>,
    stream: SlimeStream,
    fbb: FlatBufferBuilder<'static>,
}
impl M<Connected> {
    pub async fn request_feed(mut self) -> Result<M<Active>, (M<Disconnected>, WsError)> {
        use solarxr_protocol::{
            data_feed::{DataFeedMessageHeader, DataFeedMessageHeaderArgs},
            MessageBundleArgs,
        };
        let fbb = &mut self.state.fbb;
        let data = {
            let header = DataFeedMessageHeader::create(
                fbb,
                &DataFeedMessageHeaderArgs {
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
                    sink: self.state.sink,
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

pub struct Active {
    sink: Pin<SlimeSink>,
    stream: SlimeStream,
}
impl M<Active> {
    pub async fn recv(mut self) -> Result<(Self, FeedUpdate), (M<Disconnected>, eyre::Report)> {
        let result = self.state.stream.next().await;
        let result = match result {
            Some(Ok(v)) => Ok(FeedUpdate(v)),
            Some(Err(err)) => Err(err).wrap_err("Websocket error while receiving"),
            None => Err(eyre!("Stream was `None`")),
        };
        match result {
            Ok(d) => Ok((self, d)),
            Err(err) => Err((self.into_state(Disconnected), err)),
        }
    }
}

pub enum RecvResult {
    Ok(M<Connected>, DataResult),
    NoData(M<Connected>),
    CriticalError(M<Disconnected>, WsError),
}
