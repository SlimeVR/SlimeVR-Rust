mod client;
mod codec;
mod color;
mod data;
mod model;

pub use color::RGBA;

use std::f32::consts::PI;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc};
use std::time::Duration;

use eyre::{Result, WrapErr};

use nalgebra::{Isometry3, SVector, UnitQuaternion, Vector3};
use ovr_overlay as ovr;

use crate::model::skeleton::SkeletonBuilder;
use crate::model::BoneKind;

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

    let mut iso = Isometry3 {
        translation: SVector::from([0., 0., 0.]).into(),
        ..Default::default()
    };

    // Set up overlay
    let mut skeleton = SkeletonBuilder::default()
        .build(mngr)
        .wrap_err("Could not create skeleton")?;
    skeleton.set_visibility(true);

    log::info!("Main Loop");
    let start_time = std::time::SystemTime::now();
    while !stop_signal.load(Ordering::Relaxed) {
        let elapsed = start_time.elapsed().unwrap().as_secs_f32();

        let rotation =
            UnitQuaternion::from_axis_angle(&Vector3::x_axis(), elapsed * ROTATION_SPEED);
        iso.rotation = rotation;
        iso.translation.vector = SVector::from([(elapsed * TRANSLATION_SPEED).sin(), 0., 0.]);
        for bone_kind in BoneKind::iter() {
            skeleton.set_isometry(bone_kind, iso);
            skeleton.set_length(bone_kind, ((elapsed * SIZE_SPEED).cos() + 1.0) * 0.5);
            if let Err(e) = skeleton.update_render(bone_kind, mngr) {
                log::error!("Error updating render for bone {bone_kind:?}: {}", e);
            }
        }

        std::thread::sleep(Duration::from_millis(1));
    }

    log::info!("Shutting down OpenVR context");
    unsafe { context.shutdown() };
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
