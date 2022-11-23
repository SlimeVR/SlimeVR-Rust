pub mod prelude;
pub mod settings;
pub mod topic;

mod data;
mod data_feed;
mod pub_sub;
mod state_machine;

pub use solarxr_protocol as protocol;

pub use crate::data::{Data, DecodeError, FeedUpdate};
use crate::data_feed::DataFeedCallback;
use crate::state_machine::{ClientStateMachine, DeserializeError, RecvError};

use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

type Wss = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Returns a future that will run forever, continually calling the callbacks as necessary
pub async fn run<Fut>(
	connect_to: String,
	mut data_feed_callback: impl DataFeedCallback,
) -> ! {
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
					let bundle = update.0.table();
					if let Some(msgs) = bundle.data_feed_msgs() {
						self::data_feed::handle_data_feed(
							&mut data_feed_callback,
							msgs.into_iter(),
						)
						.await
					}
					if let Some(msgs) = bundle.pub_sub_msgs() {
						self::pub_sub::handle_pub_sub(msgs.into_iter()).await
					}
					active = Some(a);
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
