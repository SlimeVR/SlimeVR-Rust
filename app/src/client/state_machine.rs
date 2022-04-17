use crate::client::{Data, DataResult};

use eyre::{eyre, Result, WrapErr};
use futures_util::stream::SplitStream;
use futures_util::{Sink, SinkExt, StreamExt};
use tokio::sync::watch;
use tokio_tungstenite::{connect_async, tungstenite};
use tungstenite::error::Error as WsError;
use tungstenite::Message;

use super::data::FeedUpdate;
use super::Wss;

type DeserializeFn = fn(Result<Message, WsError>) -> Result<Data>;
type SlimeStream = futures_util::stream::Map<SplitStream<Wss>, DeserializeFn>;
type SlimeSink = Box<dyn Sink<Data, Error = eyre::Report>>; // Cringe

pub struct ClientStateMachine<State = Disconnected> {
    state: State,
    // Data common to all states goes here
    connect_to: String,
}
impl ClientStateMachine {
    /// Creates a new `NetworkStateMachine`. This starts in the [`Disconnected`] state.
    pub fn new(connect_to: String) -> Self {
        Self {
            state: Disconnected,
            connect_to,
        }
    }
}
impl<S> ClientStateMachine<S> {
    /// Helper function to transition to next state while preserving all common data
    fn into_state<Next>(self, state: Next) -> ClientStateMachine<Next> {
        ClientStateMachine {
            connect_to: self.connect_to,
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
        match connect_async(&self.connect_to.clone())
            .await
            .wrap_err("Could not open websocket connection")
        {
            Ok((socket, _)) => {
                let (sink, stream) = socket.split();
                async fn serialize(data: Data) -> Result<Message> {
                    let v = data.serialize();
                    Ok(Message::Binary(v))
                }
                fn deserialize(msg: Result<Message, WsError>) -> Result<Data> {
                    match msg {
                        Ok(Message::Binary(v)) => {
                            Ok(Data::deserialize(v).wrap_err("Invalid message")?)
                        }
                        _ => Err(eyre!("Invalid websocket payload type")),
                    }
                }
                let sink = Box::new(sink.with(serialize));
                let stream: SlimeStream = stream.map(deserialize);
                Ok(self.into_state(Connected { sink, stream }))
            }
            Err(e) => Err((self.into_state(Disconnected), e)),
        }
    }
}

/// Client is connected over websocket
pub struct Connected {
    sink: SlimeSink,
    stream: SlimeStream,
}
impl M<Connected> {
    pub async fn request_feed(self) -> Result<M<Active>, (M<Disconnected>, WsError)> {
        todo!()
    }
}

pub struct Active {
    sink: SlimeSink,
    stream: SlimeStream,
    sender: watch::Sender<FeedUpdate>,
}
impl M<Active> {
    pub async fn recv(self) -> Result<(Self, FeedUpdate), (M<Disconnected>, WsError)> {
        todo!()
    }
}

pub enum RecvResult {
    Ok(M<Connected>, DataResult),
    NoData(M<Connected>),
    CriticalError(M<Disconnected>, WsError),
}
