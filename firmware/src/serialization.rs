use embassy_time::{Duration, Ticker};
use futures_util::StreamExt;

use crate::{
	imu::Quat,
	networking::{messaging::Signal, Message, Signals},
};

pub async fn serialize_task(msg_signals: &Signals, _quat_signal: &Signal<Quat>) {
	let mut ticker = Ticker::every(Duration::from_millis(1000));
	loop {
		ticker.next().await;
		msg_signals.latest.signal(Message);
	}
}
