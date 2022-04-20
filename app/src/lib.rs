mod client;
mod color;
mod model;

pub use color::RGBA;

use crate::model::skeleton::SkeletonBuilder;
use crate::model::BoneKind;

use eyre::{Result, WrapErr};
use lazy_static::lazy_static;
use nalgebra::{Isometry3, SVector, UnitQuaternion, Vector3};
use ovr_overlay as ovr;
use std::f32::consts::PI;
use std::time::Duration;
use tokio_graceful_shutdown::{SubsystemHandle, Toplevel};

const ROTATION_SPEED: f32 = 2.0 * 2.0 * PI;
const TRANSLATION_SPEED: f32 = 0.5 * 2.0 * PI;
const SIZE_SPEED: f32 = 0.25 * 2.0 * PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShutdownReason {
    CtrlC,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    Toplevel::new()
        .start("Overlay", overlay)
        .catch_signals()
        .handle_shutdown_requests(Duration::from_millis(1000))
        .await
}

pub async fn overlay(subsys: SubsystemHandle) -> Result<()> {
    log::info!("Initializing OpenVR context");
    let context = ovr::Context::init().wrap_err("Failed to initialize OpenVR")?;
    let mngr = &mut context.overlay_mngr();

    let mut iso = Isometry3 {
        translation: SVector::from([0., 0., 0.]).into(),
        ..Default::default()
    };

    let mut skeleton = SkeletonBuilder::default()
        .build(mngr)
        .wrap_err("Could not create skeleton")?;
    skeleton.set_visibility(true);

    log::info!("Main Loop");
    let start_time = std::time::SystemTime::now();
    let result = tokio::select! {
        _ = subsys.on_shutdown_requested() => {
            log::debug!("overlay shutdown requested");
        },
        _ = async {
            loop {
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

                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        } => unreachable!(),
    };

    log::info!("Shutting down OpenVR context");
    unsafe { context.shutdown() };
    Ok(())
}
