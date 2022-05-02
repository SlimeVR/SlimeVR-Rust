mod client;
mod color;
mod model;

pub use self::color::RGBA;

use crate::client::{Client, FeedUpdate};
use crate::model::skeleton::SkeletonBuilder;
use crate::model::BoneKind;

use eyre::{Result, WrapErr};
use nalgebra::{Isometry3, SVector, UnitQuaternion, Vector3};
use ovr_overlay as ovr;
use std::f32::consts::PI;
use std::time::Duration;
use tokio::sync::watch;
use tokio_graceful_shutdown::{SubsystemHandle, Toplevel};

const ROTATION_SPEED: f32 = 2.0 * 2.0 * PI;
const TRANSLATION_SPEED: f32 = 0.5 * 2.0 * PI;
const SIZE_SPEED: f32 = 0.25 * 2.0 * PI;

const CONNECT_STR: &'static str = "ws://localhost:21110";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShutdownReason {
    CtrlC,
}

macro_rules! unwrap_or_continue {
    ($e:expr) => {{
        if let Some(inner) = $e {
            inner
        } else {
            continue;
        }
    }};
}

#[tokio::main]
pub async fn main() -> Result<()> {
    Toplevel::new()
        .start("Networking", networking)
        .catch_signals()
        .handle_shutdown_requests(Duration::from_millis(1000))
        .await
}

async fn overlay(
    mut recv: watch::Receiver<Option<FeedUpdate>>,
    subsys: SubsystemHandle,
) -> Result<()> {
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

    let loop_ = async {
        loop {
            recv.changed()
                .await
                .wrap_err("Error while attempting to watch for feed update")?;

            {
                let guard = recv.borrow_and_update();
                let table = guard.as_ref().unwrap().0.table();
                log::trace!("update: {:#?}", table);

                let m = unwrap_or_continue!(table.data_feed_msgs());
                let m = m.get(0);
                let m = unwrap_or_continue!(m.message_as_data_feed_update());
                let trackers = unwrap_or_continue!(m.synthetic_trackers());
                log::debug!("Got {} trackers before filtering", trackers.len());

                let trackers: Vec<_> = trackers
                    .iter()
                    .filter_map(|t| {
                        let part = t.info()?.body_part();
                        log::trace!("body_part: {part:?}");
                        let part = BoneKind::try_from(part).ok();

                        part
                    })
                    .collect();
                log::info!("trackers: {trackers:?}");
            }
            let elapsed = start_time.elapsed().unwrap().as_secs_f32();

            let rotation =
                UnitQuaternion::from_axis_angle(&Vector3::x_axis(), elapsed * ROTATION_SPEED);
            iso.rotation = rotation;
            iso.translation.vector = SVector::from([(elapsed * TRANSLATION_SPEED).sin(), 0., 0.]);
            for bone_kind in BoneKind::iter() {
                skeleton.set_isometry(bone_kind, iso);
                skeleton.set_length(bone_kind, ((elapsed * SIZE_SPEED).cos() + 1.0) * 0.5);
                if let Err(e) = skeleton.update_render(bone_kind, mngr) {
                    log::error!("Error updating render for bone {bone_kind:?}: {:?}", e);
                }
            }

            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    };
    tokio::select! {
        _ = subsys.on_shutdown_requested() => {
            log::debug!("overlay shutdown requested");
            Ok::<_, eyre::Report>(())
        },
        r = loop_ => r,
    }?;

    log::info!("Shutting down OpenVR context");
    unsafe { context.shutdown() };
    Ok(())
}

async fn networking(subsys: SubsystemHandle) -> Result<()> {
    let (client, recv) =
        Client::new(CONNECT_STR.to_string(), subsys.clone()).wrap_err("Failed to start client")?;
    subsys.start("Overlay", |s| overlay(recv, s));
    client.join().await
}
