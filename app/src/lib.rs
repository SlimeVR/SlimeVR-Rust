mod client;
mod color;
mod model;

pub use self::color::RGBA;

use crate::client::{Client, FeedUpdate};
use crate::model::skeleton::SkeletonBuilder;
use crate::model::{BoneKind, Isometry};

use eyre::{Result, WrapErr};
use nalgebra::{Translation3, UnitQuaternion};
use ovr_overlay as ovr;
use std::time::Duration;
use tokio::sync::watch;
use tokio_graceful_shutdown::{SubsystemHandle, Toplevel};

const CONNECT_STR: &str = "ws://localhost:21110";

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

    let mut skeleton = SkeletonBuilder::default()
        .build(mngr)
        .wrap_err("Could not create skeleton")?;
    skeleton.set_visibility(true);

    log::info!("Overlay Loop");

    let loop_ = async {
        loop {
            recv.changed()
                .await
                .wrap_err("Error while attempting to watch for feed update")?;

            log::trace!("Got a feed update");
            let bones: Vec<_> = {
                let guard = recv.borrow_and_update();
                let table = guard.as_ref().unwrap().0.table();
                log::trace!("update: {:#?}", table);

                let m = unwrap_or_continue!(table.data_feed_msgs());
                let m = m.get(0);
                let m = unwrap_or_continue!(m.message_as_data_feed_update());
                let bones = unwrap_or_continue!(m.bones());
                log::debug!("Got {} bones before filtering", bones.len());

                bones
                    .iter()
                    .filter_map(|b| {
                        let part = b.body_part();
                        log::trace!("body_part: {part:?}");
                        let bone_kind = BoneKind::try_from(part)
                            .map_err(|e| {
                                log::trace!("Filtering out {e:?}");
                                e
                            })
                            .ok()?;
                        let pos = if let Some(p) = b.head_position_g() {
                            p
                        } else {
                            log::warn!("No position");
                            return None;
                        };
                        let rot = if let Some(r) = b.rotation_g() {
                            r
                        } else {
                            log::warn!("No rotation");
                            return None;
                        };
                        let length = b.bone_length();

                        let pos = Translation3::new(pos.x(), pos.y(), pos.z());
                        let rot = UnitQuaternion::from_quaternion(
                            [rot.x(), rot.y(), rot.z(), rot.w()].into(),
                        );
                        Some((bone_kind, pos, rot, length))
                    })
                    .collect()
            };
            log::debug!(
                "Bones after filtering: {:?}",
                bones.iter().map(|t| t.0).collect::<Vec<_>>()
            );
            log::trace!("Bone data: {bones:?}");
            for (bone_kind, pos, rot, length) in bones {
                let iso = Isometry {
                    rotation: rot,
                    translation: pos,
                };
                skeleton.set_isometry(bone_kind, iso);
                skeleton.set_length(bone_kind, length);
                if let Err(e) = skeleton.update_render(bone_kind, mngr) {
                    log::error!("Error updating render for bone {bone_kind:?}: {:?}", e);
                }
            }
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
