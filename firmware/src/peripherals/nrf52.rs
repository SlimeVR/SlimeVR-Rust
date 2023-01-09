use super::Peripherals;
use crate::aliases::ඞ::DelayConcrete;
use crate::aliases::ඞ::I2cConcrete;
use crate::aliases::ඞ::UartConcrete;
use crate::aliases::ඞ::UsbDriverConcrete;

use defmt::debug;
use embassy_nrf::interrupt;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::uarte::{self, Uarte};
use paste::paste;

macro_rules! map_pin {
	($io: ident, $pin: expr) => {
		paste! {
			$io.[<P $pin>]
		}
	};
}

pub fn get_peripherals() -> Peripherals<
	I2cConcrete<'static>,
	DelayConcrete,
	UartConcrete<'static>,
	UsbDriverConcrete<'static>,
> {
	let p = embassy_nrf::init(Default::default());

	// Fix issue on rev 3 boards where AP is protected, preventing debugging/rtt.
	#[cfg(feature = "mcu-nrf52840")] // TODO: Add nrf52832 support
	{
		use defmt::{error, info};
		use nrf52840_pac as pac;

		let code = {
			// Safety: should not have any references held elsewhere
			let ficr = unsafe { &*pac::FICR::ptr() };
			ficr.info.variant.read().bits().to_be_bytes()
		};

		// Get third character of the variant to determine hardware revision
		// https://infocenter.nordicsemi.com/index.jsp?topic=%2Fps_nrf52840%2Fficr.html&cp=4_0_0_3_3_0_8&anchor=register.INFO.VARIANT
		let rev = Revision::from_variant(code);
		if let Some(rev) = rev {
			info!("Chip revision: {}", rev);
			if rev == Revision::Rev3 {
				// Set UICR.APPROTECT to HwDisable and APPROTECT.DISABLE to SwDisable
				// todo: Use an updated pac or hal to do this, when it comes out.
				// This is dangerous because of the lack of proper atomics or against race conditions.
				unsafe {
					let nvmc = &*pac::NVMC::ptr();
					let approtect = &mut (*pac::UICR::ptr().cast_mut()).approtect;
					const HW_DISABLED: u32 = 0x05a;
					const SW_DISABLED: u32 = 0x05a;

					if approtect.read().bits() != HW_DISABLED {
						nvmc.config.write(|w| w.wen().wen());
						while nvmc.ready.read().ready().is_busy() {}
						core::ptr::write_volatile(approtect.as_ptr(), HW_DISABLED);
						while nvmc.ready.read().ready().is_busy() {}
						nvmc.config.reset();
						while nvmc.ready.read().ready().is_busy() {}
						cortex_m::peripheral::SCB::sys_reset();
					}

					// APPROTECT.DISABLE = SwDisabled
					(0x4000_0558 as *mut u32).write_volatile(SW_DISABLED);
				}
			}
		} else {
			error!("Unknown hardware revision!");
		}

		/// The hardware revision of the chip.
		/// See https://devzone.nordicsemi.com/f/nordic-q-a/55614/how-to-apply-nordic-software-workarounds-errata-for-a-given-hardware-revision-in-the-field
		#[derive(defmt::Format, Eq, PartialEq, Copy, Clone)]
		#[non_exhaustive]
		enum Revision {
			EngA,
			EngB,
			EngC,
			EngD,
			Rev1,
			Rev2,
			Rev3,
		}
		impl Revision {
			fn from_variant(bytes: [u8; 4]) -> Option<Revision> {
				let prefix = bytes[2];
				let suffix = bytes[3];

				let digit = suffix.is_ascii_digit();
				Some(match prefix {
					b'A' => Self::EngA,
					b'B' => Self::EngB,
					b'C' if !digit => Self::EngC,
					b'D' if !digit => Self::EngD,
					b'C' if digit => Self::Rev1,
					b'D' if digit => Self::Rev2,
					b'F' if digit => Self::Rev3,
					_ => return None,
				})
			}
		}
	}

	debug!("Initializing TWIM (I2C controller)");

	// IDK how this works, code is from here:
	// https://github.com/embassy-rs/embassy/blob/f109e73c6d7ef2ad93102b7c8223f5cef30ef36f/examples/nrf/src/bin/twim.rs
	let twim = {
		let config = twim::Config::default();
		let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
		Twim::new(
			p.TWISPI0,
			irq,
			map_pin!(p, env!("SDA_PIN")),
			map_pin!(p, env!("SCL_PIN")),
			config,
		)
	};
	debug!("Initialized twim");

	let delay = embassy_time::Delay;
	debug!("Initialized delay");

	let uarte = {
		let irq = interrupt::take!(UARTE0_UART0);
		let mut config = uarte::Config::default();
		config.parity = uarte::Parity::EXCLUDED;
		config.baudrate = uarte::Baudrate::BAUD115200;
		let tx = map_pin!(p, env!("TX_PIN"));
		let rx = map_pin!(p, env!("RX_PIN"));

		Uarte::new(p.UARTE0, irq, rx, tx, config)
	};
	debug!("Initialized uarte");

	#[allow(unused_variables)]
	let usb_driver = ();
	#[cfg(feature = "mcu-nrf52840")]
	let usb_driver = {
		use embassy_nrf::usb::{self, Driver};
		let irq = interrupt::take!(USBD);
		let power_irq = interrupt::take!(POWER_CLOCK);
		let d = Driver::new(p.USBD, irq, usb::PowerUsb::new(power_irq));
		debug!("Initialized usb_driver");
		d
	};

	let p = Peripherals::new();
	p.i2c(twim).delay(delay).uart(uarte).usb_driver(usb_driver)
}
