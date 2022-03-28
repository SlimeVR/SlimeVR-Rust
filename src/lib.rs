mod bone;

use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc};
use std::time::Duration;

use eyre::{Result, WrapErr};

use ovr_overlay as ovr;

use crate::bone::{Bone, Isometry};

const RADIUS: f32 = 0.01;

pub fn main() -> Result<()> {
    let stop_signal = Arc::new(AtomicBool::new(false));

    {
        let stop_signal_copy = stop_signal.clone();
        ctrlc::set_handler(move || stop_signal_copy.store(true, Ordering::Relaxed)).unwrap();
    }

    log::info!("Initializing OpenVR context");
    let context = ovr::Context::init().wrap_err("Failed to initialize OpenVR")?;
    let mngr = &mut context.overlay_mngr();

    let mut bone_length = RADIUS * 2.;

    // Set up overlay
    let mut bone = Bone::new(
        mngr,
        RGBA {
            r: 255,
            g: 255,
            b: 0,
            a: 20,
        },
        Isometry::default(),
        String::from("Bone"),
        RADIUS,
        bone_length,
    )
    .wrap_err("Could not create bone")?;

    bone.set_visibility(true);

    log::info!("Main Loop");
    while !stop_signal.load(Ordering::Relaxed) {
        bone_length *= 1.5;
        bone.set_length(bone_length);
        bone.update_render(mngr)
            .wrap_err("Could not update render")?;
        std::thread::sleep(Duration::from_millis(1000));
    }

    log::info!("Shutting down OpenVR context");
    unsafe { context.shutdown() };
    Ok(())
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RGBA {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
