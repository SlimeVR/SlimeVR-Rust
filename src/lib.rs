mod bone;

use std::f32::consts::PI;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc};
use std::time::Duration;

use eyre::{Result, WrapErr};

use nalgebra::{Isometry3, SVector, UnitQuaternion, Vector3};
use ovr_overlay as ovr;

use crate::bone::Bone;

const RADIUS: f32 = 0.01;

const ROTATION_SPEED: f32 = 2.0 * 2.0 * PI;
const TRANSLATION_SPEED: f32 = 0.5 * 2.0 * PI;
const SIZE_SPEED: f32 = 0.25 * 2.0 * PI;

pub fn main() -> Result<()> {
    let stop_signal = Arc::new(AtomicBool::new(false));

    {
        let stop_signal_copy = stop_signal.clone();
        ctrlc::set_handler(move || stop_signal_copy.store(true, Ordering::Relaxed)).unwrap();
    }

    log::info!("Initializing OpenVR context");
    let context = ovr::Context::init().wrap_err("Failed to initialize OpenVR")?;
    let mngr = &mut context.overlay_mngr();

    let bone_length = 1.;
    let mut iso = Isometry3 {
        translation: SVector::from([0., 0., 0.]).into(),
        ..Default::default()
    };

    // Set up overlay
    let mut bone = Bone::new(
        mngr,
        RGBA {
            r: 255,
            g: 255,
            b: 0,
            a: 20,
        },
        iso,
        String::from("Bone"),
        RADIUS,
        bone_length,
    )
    .wrap_err("Could not create bone")?;

    bone.set_visibility(true);

    log::info!("Main Loop");
    let start_time = std::time::SystemTime::now();
    while !stop_signal.load(Ordering::Relaxed) {
        let elapsed = start_time.elapsed().unwrap().as_secs_f32();

        let rotation =
            UnitQuaternion::from_axis_angle(&Vector3::x_axis(), elapsed * ROTATION_SPEED);
        iso.rotation = rotation;
        iso.translation.vector = SVector::from([(elapsed * TRANSLATION_SPEED).sin(), 0., 0.]);
        bone.set_isometry(iso);
        bone.set_length(((elapsed * SIZE_SPEED).cos() + 1.0) * 0.5);
        bone.update_render(mngr)
            .wrap_err("Could not update render")?;
        std::thread::sleep(Duration::from_millis(1));
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
