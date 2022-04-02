mod state_machine;

use crate::data::Data;

use eyre::eyre;
use eyre::{Result, WrapErr};
use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio::sync;
use tokio::sync::oneshot;
use tokio::task;
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

type Wss = WebSocketStream<MaybeTlsStream<TcpStream>>;

type DisconnectReason = eyre::Report;

pub struct Client {
    socket_task: JoinHandle<Result<()>>,
    shutdown_signal: sync::oneshot::Sender<()>,
}
impl Client {
    pub async fn new<C>(connect_to: C) -> Result<(Self, sync::mpsc::Receiver<Data>)>
    where
        C: IntoClientRequest + Clone + Unpin + Send + Sync + 'static,
    {
        let (shutdown_signal, shutdown_signal_recv) = sync::oneshot::channel();
        let (data_send, data_recv) = sync::mpsc::channel(1);

        let socket_task = task::spawn(async move {
            let mut shutdown_signal = shutdown_signal_recv;
            loop {
                tokio::select! {
                    _ = &mut shutdown_signal => {
                        break;
                    }
                    else => {
                        let (socket, _) = connect_async(connect_to.clone())
                            .await.wrap_err("Could not open websocket connection")?;
                        if let Err(err) = Self::on_connect(socket, &mut shutdown_signal, data_send.clone()).await {
                            Self::on_disconnect(err)
                        }
                    }
                }
            }
            Ok(())
        });

        Ok((
            Self {
                socket_task,
                shutdown_signal,
            },
            data_recv,
        ))
    }

    async fn on_connect(
        socket: Wss,
        mut stop_signal: &mut oneshot::Receiver<()>,
        mut data_send: sync::mpsc::Sender<Data>,
    ) -> Result<(), DisconnectReason> {
        let (mut sink, mut stream): (SplitSink<Wss, Message>, SplitStream<Wss>) = socket.split();
        // Do initial handshake
        {
            //TODO
        }
        loop {
            tokio::select! {
                _ = &mut stop_signal => {
                    return Err(eyre!("Stopped by us"));
                },
                msg = stream.next() => {
                    match msg {
                        Some(Ok(msg)) => Self::on_msg(&mut sink, &mut data_send, msg)?,
                        Some(Err(err)) => return Err(err).wrap_err("Tungstenite error")?,
                        None => {
                            log::warn!("Remote disconnected!")
                        }
                    }
                },
            }
        }
    }

    fn on_msg(
        sink: &mut SplitSink<Wss, Message>,
        data_send: &mut sync::mpsc::Sender<Data>,
        msg: Message,
    ) -> Result<()> {
        log::debug!("{}", msg);
        let v = msg.into_data();
        let data = Data::new_from_data(v).wrap_err("Failed to deserialize message")?;
        data_send.try_send(data).ok(); // If we would block, better to return and fetch again
        Ok(())
    }

    fn on_disconnect(err: DisconnectReason) {
        // TODO: Actually handle this
        log::error!("{}", err);
    }

    pub async fn shutdown(self) -> Result<()> {
        drop(self.shutdown_signal);
        self.socket_task.await.wrap_err("Failed to join!")?
    }

    pub async fn join(self) -> Result<()> {
        self.socket_task.await.wrap_err("Failed to join!")?
    }
}
