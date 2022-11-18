mod data;
pub mod settings;
mod state_machine;
pub mod topic;

pub use solarxr_protocol as protocol;

pub use crate::data::{Data, DecodeError, FeedUpdate};
use crate::state_machine::{ClientStateMachine, DeserializeError, RecvError};

use core::future::Future;
use eyre::Result;
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

type Wss = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub async fn run<Fut>(
	connect_to: String,
	data_feed_callback: impl Fn(FeedUpdate) -> Fut,
) -> Result<()>
where
	Fut: Future<Output = ()>,
{
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
					RecvError::None(d) => (d, eyre::eyre!("Stream produced `None`")),
					RecvError::NoTopicMapping(d) => {
						(d, eyre::eyre!("Failed to get a topic mapping"))
					}
				};

				log::error!("{:?}", err.wrap_err("Error while requesting feed"));
				disconnected = Some(d);
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
					data_feed_callback(update).await;
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
