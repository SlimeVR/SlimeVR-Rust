use embassy_time::{Duration, Timer};

use crate::{aliases::ඞ::NetConcrete, networking::protocol::Packets};

pub async fn network_task(_packets: &Packets, _net: NetConcrete) -> ! {
	loop {
		Timer::after(Duration::from_millis(10_000)).await
	}
}
