use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc};
use std::time::Duration;

use eyre::{Result, WrapErr};

use ovr_overlay as ovr;

const OVERLAY_KEY: &'static str = "SLIMEVR_OVERLAY";
const OVERLAY_DISPLAY_NAME: &'static str = "SlimeVR Overlay";

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

fn main() -> Result<()> {
    pretty_env_logger::init();

    let stop_signal = Arc::new(AtomicBool::new(false));

    {
        let stop_signal_copy = stop_signal.clone();
        ctrlc::set_handler(move || stop_signal_copy.store(true, Ordering::Relaxed)).unwrap();
    }

    log::info!("Initializing OpenVR context");
    let context = ovr::Context::init().wrap_err("Failed to initialize OpenVR")?;
    let mut overlay = context.overlay();

    // Set up overlay
    let _handle = {
        let handle = overlay
            .create_overlay(OVERLAY_KEY, OVERLAY_DISPLAY_NAME)
            .wrap_err("Failed to create overlay")?;

        // Log some info about the overlay state
        log::debug!("Visible: {}", overlay.is_visible(handle));
        log::debug!("Opacity: {}", overlay.opacity(handle)?);
        log::debug!("Curvature: {}", overlay.curvature(handle)?);
        log::debug!("Width: {}", overlay.width(handle)?);

        overlay.set_curvature(handle, -1.)?;
        let white_pixels = vec![255; WIDTH * HEIGHT * 4];
        overlay
            .set_raw_data(handle, white_pixels, WIDTH, HEIGHT, 4)
            .wrap_err("Failed to set raw data")?;
        overlay
            .show_overlay(handle)
            .wrap_err("Failed to show overlay")?;

        handle
    };

    log::info!("Main Loop");
    while !stop_signal.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(1));
    }

    log::info!("Shutting down OpenVR context");
    unsafe { context.shutdown() };
    Ok(())
}
