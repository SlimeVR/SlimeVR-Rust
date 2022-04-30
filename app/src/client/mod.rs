mod data;
mod state_machine;

use self::data::{Data, DataResult, FeedUpdate};
use self::state_machine::ClientStateMachine;

use eyre::{Result, WrapErr};

use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio::task;
use tokio::task::JoinHandle;
use tokio_graceful_shutdown::SubsystemHandle;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

type Wss = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct Client {
    socket_task: JoinHandle<Result<()>>,
}
impl Client {
    pub fn new(
        connect_to: String,
        sub: SubsystemHandle,
    ) -> Result<(Self, watch::Receiver<Option<FeedUpdate>>)> {
        let (data_send, data_recv) = watch::channel(None);

        let socket_task = task::spawn(async move {
            tokio::select! {
                _ = sub.on_shutdown_requested() => {}
                result = Self::run(connect_to, data_send) => {result?}
            };
            Ok(())
        });

        Ok((Self { socket_task }, data_recv))
    }

    async fn run(connect_to: String, data_send: watch::Sender<Option<FeedUpdate>>) -> Result<()> {
        let mut disconnected = Some(ClientStateMachine::new(connect_to));
        loop {
            let ready = match disconnected.take().unwrap().connect().await {
                Ok(ready) => ready,
                Err((d, err)) => {
                    log::error!("{:?}", err.wrap_err("Failed to connect"));
                    disconnected = Some(d);
                    continue;
                }
            };
            let active = match ready.request_feed().await {
                Ok(active) => active,
                Err((d, err)) => {
                    log::error!(
                        "{:?}",
                        eyre::Report::new(err).wrap_err("Failed to request feed")
                    );
                    disconnected = Some(d);
                    continue;
                }
            };
            let mut active = Some(active);
            loop {
                match active.take().unwrap().recv().await {
                    Ok((a, update)) => {
                        active = Some(a);
                        data_send.send_replace(Some(update));
                    }
                    Err((d, err)) => {
                        log::error!("{:?}", err.wrap_err("Failed to receive feed"));
                        disconnected = Some(d);
                        break;
                    }
                }
            }
        }
    }

    pub async fn join(self) -> Result<()> {
        self.socket_task.await.wrap_err("Failed to join!")?
    }
}
