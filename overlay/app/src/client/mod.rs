mod data;
mod state_machine;
pub mod topic;

use crate::client::state_machine::DeserializeError;

pub use self::data::{Data, DecodeError, FeedUpdate};
use self::state_machine::{ClientStateMachine, RecvError};

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

	async fn run(
		connect_to: String,
		data_send: watch::Sender<Option<FeedUpdate>>,
	) -> Result<()> {
		let mut disconnected = Some(ClientStateMachine::new(connect_to));
		loop {
			let ready = match disconnected.take().unwrap().connect().await {
				Ok(ready) => ready,
				Err((d, err)) => {
					log::error!("Error while connecting: {}", err);
					disconnected = Some(d);
					continue;
				}
			};
			let active = match ready.request_feed().await {
				Ok(active) => active,
				Err(err) => {
					let (d, err) = match err {
						RecvError::CriticalWs(d, err) => (d, eyre::Report::new(err)),
						RecvError::Deserialize(a, err) => {
							let d = a.into_state(self::state_machine::Disconnected);
							(d, eyre::Report::new(err))
						}
						RecvError::None(d) => {
							(d, eyre::eyre!("Stream produced `None`"))
						}
						RecvError::NoTopicMapping(d) => {
							(d, eyre::eyre!("Failed to get a topic mapping"))
						}
					};

					log::error!("{:?}", err.wrap_err("Error while requesting feed"));
					continue;
				}
			};
			let mut active = Some(active);
			loop {
				use RecvError as E;
				match active.take().unwrap().recv().await {
					Ok((a, update)) => {
						log::trace!("Sending data to watchers: {:#?}", update);
						active = Some(a);
						data_send.send_replace(Some(update));
					}
					Err(err) => {
						let display = format!("{}", &err);
						match err {
							E::CriticalWs(d, _) => {
								log::error!("Critical websocket error: {}", display);
								disconnected = Some(d);
								break;
							}
							E::None(d) => {
								log::error!("Critical websocket error: {}", display);
								disconnected = Some(d);
								break;
							}
							E::Deserialize(a, DeserializeError::PayloadType(_)) => {
								active = Some(a)
							}
							E::Deserialize(a, d_err) => {
								match d_err {
									DeserializeError::PayloadType(_) => {
										log::trace!("{}", d_err)
									}
									_ => {
										log::warn!("Deserialization error: {}", display)
									}
								}
								active = Some(a);
							}
							E::NoTopicMapping(_) => unreachable!(
								"Topic mapping only relevant in Connected state"
							),
						}
					}
				}
			}
		}
	}

	pub async fn join(self) -> Result<()> {
		self.socket_task.await.wrap_err("Failed to join!")?
	}
}
