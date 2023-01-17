//! The protocol implementation to communicate with the SlimeVR Server.

mod packets;
pub use self::packets::Packets;

use defmt::debug;
use embassy_executor::task;
use embassy_futures::select::select;
use firmware_protocol::{
	sansio::{self, SbBuf},
	CbPacket, SlimeQuaternion,
};

use crate::imu::Quat;
use crate::utils::{Reliable, Unreliable};

#[task]
pub async fn protocol_task(
	packets: &'static Packets,
	quat: &'static Unreliable<Quat>,
) -> ! {
	debug!("Control task!");
	async {
		let mut proto_state = sansio::State::new();
		let mut sb_buf = SbBuf::new();
		loop {
			proto_state = match proto_state {
				sansio::State::Disconnected(s) => {
					while_disconnected(s, &mut sb_buf, &packets.clientbound).await
				}
				sansio::State::Connected(s) => {
					while_connected(s, &mut sb_buf, &packets.clientbound, quat).await
				}
			};
			while !sb_buf.is_empty() {
				let sb = sb_buf.pop().unwrap();
				packets.serverbound.send(sb).await;
			}
		}
	}
	.await
}

async fn while_disconnected(
	state: sansio::Disconnected,
	sb_buf: &mut SbBuf,
	cb: &Reliable<CbPacket>,
) -> sansio::State {
	let cb_msg = cb.recv().await;
	state.received_msg(cb_msg, sb_buf)
}

async fn while_connected(
	state: sansio::Connected,
	sb_buf: &mut SbBuf,
	cb: &Reliable<CbPacket>,
	quat: &Unreliable<Quat>,
) -> sansio::State {
	let event = select(cb.recv(), quat.wait()).await;
	use embassy_futures::select::Either;
	match event {
		Either::First(cb_msg) => state.received_msg(cb_msg, sb_buf),
		Either::Second(quat) => state.send_imu(0, SlimeQuaternion::from(quat), sb_buf),
	}
}
