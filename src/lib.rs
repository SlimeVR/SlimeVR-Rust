mod bone;

use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc};
use std::time::Duration;

use eyre::{Result, WrapErr};

use ovr_overlay as ovr;

use crate::bone::{Bone, Isometry};

const OVERLAY_KEY: &'static str = "SLIMEVR_OVERLAY";
const OVERLAY_DISPLAY_NAME: &'static str = "SlimeVR Overlay";

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const RADIUS: f32 = 0.05;

pub fn main() -> Result<()> {
    let stop_signal = Arc::new(AtomicBool::new(false));

    {
        let stop_signal_copy = stop_signal.clone();
        ctrlc::set_handler(move || stop_signal_copy.store(true, Ordering::Relaxed)).unwrap();
    }

    log::info!("Initializing OpenVR context");
    let context = ovr::Context::init().wrap_err("Failed to initialize OpenVR")?;
    let mngr = &mut context.overlay_mngr();

    // Set up overlay
    let bone = Bone::new(
        mngr,
        RGBA {
            red: 255,
            green: 255,
            blue: 0,
            alpha: 20,
        },
        Isometry::default(),
        String::from("Bone"),
        RADIUS,
        1.,
    )
    .wrap_err("Could not create bone")?;
    bone.set_visibility(mngr, true)?;

    log::info!("Main Loop");
    while !stop_signal.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(1));
    }

    log::info!("Shutting down OpenVR context");
    unsafe { context.shutdown() };
    Ok(())
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RGBA {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
