use super::Wss;
use crate::data::{Data, DataResult};

use eyre::{Result, WrapErr};
use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::{connect_async, tungstenite};
use tungstenite::error::Error as WsError;
use tungstenite::Message;

pub struct ClientStateMachine<State = Disconnected> {
    state: State,
    // Data common to all states goes here
    connect_to: String,
}
impl ClientStateMachine {
    /// Creates a new `NetworkStateMachine`. This starts in the [`Disconnected`] state.
    pub fn new(connect_to: String, sender: Sender<Data>) -> Self {
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
    pub async fn connect(self) -> Result<M<ReadyForHandshake>, (Self, eyre::Report)> {
        match connect_async(&self.connect_to.clone())
            .await
            .wrap_err("Could not open websocket connection")
        {
            Ok((socket, _)) => Ok(self.into_state(ReadyForHandshake { socket })),
            Err(e) => Err((self.into_state(Disconnected), e)),
        }
    }
}

/// Client is connected over websocket but not yet handshaked.
pub struct ReadyForHandshake {
    socket: Wss,
}
impl M<ReadyForHandshake> {
    pub async fn handshake(self) -> Result<M<Connected>, (Self, eyre::Report)> {
        todo!()
    }
}

/// Client is connected and ready to send data
pub struct Connected {
    sink: SplitSink<Wss, Message>,
    stream: SplitStream<Wss>,
}
impl M<Connected> {
    /// Receives data
    pub async fn recv(mut self) -> RecvResult {
        if let Some(stream_result) = self.state.stream.next().await {
            match stream_result {
                Ok(msg) => RecvResult::Ok(self, Data::new_from_data(msg.into_data())),
                Err(err) => RecvResult::CriticalError(self.into_state(Disconnected), err),
            }
        } else {
            RecvResult::NoData(self)
        }
    }
}

pub enum RecvResult {
    Ok(M<Connected>, DataResult),
    NoData(M<Connected>),
    CriticalError(M<Disconnected>, WsError),
}
