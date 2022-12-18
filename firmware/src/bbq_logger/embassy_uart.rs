//! defmt-bbq logger using UART

use defmt::debug;
use defmt_bbq::DefmtConsumer;
use embassy_futures::yield_now;
use embassy_nrf::uarte::Error;

pub async fn logger_task(
	mut bbq: DefmtConsumer,
	mut uart: crate::aliases::ඞ::UartConcrete<'static>,
) {
	debug!("UART logger task!");

	loop {
		let Ok(grant) = bbq.read() else {
			yield_now().await;
			continue;
		};
		let len = grant.buf().len();
		let _result = uart.write(b"got data: ").await;
		match uart.write_from_ram(grant.buf()).await {
			Err(Error::DMABufferNotInDataMemory) => {
				// unreachable!("bbq should always be in RAM")
				()
			}
			Err(Error::BufferTooLong) => (),
			Ok(()) => (),
			_ => (),
		};
		grant.release(len);
	}
}
