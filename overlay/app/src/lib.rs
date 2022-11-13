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
use solarxr_protocol::pub_sub::Message;
use std::collections::HashSet;
use std::time::Duration;
use tokio::sync::watch;
use tokio_graceful_shutdown::{SubsystemHandle, Toplevel};

const CONNECT_STR: &str = "ws://localhost:21110";
const TOPIC_ORG: &str = "slimevr.dev";
const TOPIC_APP: &str = "overlay";
const TOPIC_DISPLAY_SETTINGS: &str = "display_settings";

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

	log::info!("Overlay Loop");

	let loop_ = async {
		let mut hidden_bones: HashSet<BoneKind> = HashSet::new();
		let mut is_skeleton_visible = true;
		loop {
			recv.changed()
				.await
				.wrap_err("Error while attempting to watch for feed update")?;

			log::trace!("Got a feed update");

			// Mark all bones as "need to hide"
			hidden_bones.extend(BoneKind::iter());

			#[derive(Debug)]
			struct BoneInfo {
				kind: BoneKind,
				pos: Translation3<f32>,
				rot: UnitQuaternion<f32>,
				length: f32,
			}
			// Extract relevant data about bones from flatbuffers
			let bones: Vec<BoneInfo> = {
				let guard = recv.borrow_and_update();
				let table = guard.as_ref().unwrap().0.table();
				log::trace!("update: {:#?}", table);

				is_skeleton_visible = {
					// Closure allows early return
					let is_visible = || -> Option<bool> {
						for m in table.pub_sub_msgs()? {
							let Some(m) = m.u_as_message() else {
								continue;
							};

							if !is_overlay_topic(m) {
								continue;
							}

							let Some(kv) = m.payload_as_key_values() else {
							continue;
						};

							let (Some(keys), Some(values)) = (kv.keys(), kv.values()) else {
							continue;
						};

							if keys.len() != values.len() {
								log::warn!("Keys and values were not same length!");
							}

							for i in 0..usize::min(keys.len(), values.len()) {
								let key = keys.get(i);
								let value = values.get(i);
								if key.to_lowercase() == "is_visible" {
									return Some(value.to_lowercase() == "true");
								}
							}
						}

						None
					}();
					is_visible.unwrap_or(is_skeleton_visible)
				};

				let m = unwrap_or_continue!(table.data_feed_msgs());

				// TODO: handle multiple updates?
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
						if is_skeleton_visible {
							hidden_bones.remove(&bone_kind);
						}
						Some(BoneInfo {
							kind: bone_kind,
							pos,
							rot,
							length,
						})
					})
					.collect()
			};

			log::debug!(
				"Bones after filtering: {:?}",
				bones.iter().map(|t| t.kind).collect::<Vec<_>>()
			);
			log::trace!("Bone data: {bones:?}");

			// Update all non-hidden bones
			for BoneInfo {
				kind,
				pos,
				rot,
				length,
			} in bones
			{
				let iso = Isometry {
					rotation: rot,
					translation: pos,
				};
				skeleton.set_isometry(kind, iso);
				skeleton.set_length(kind, length);
				skeleton.set_visibility(kind, true);
				if let Err(e) = skeleton.update_render(kind, mngr) {
					log::error!("Error updating render for bone {kind:?}: {:?}", e);
				}
			}
			// Hide all hidden bones
			for bone_kind in hidden_bones.iter() {
				skeleton.set_visibility(*bone_kind, false);
				if let Err(e) = skeleton.update_render(*bone_kind, mngr) {
					log::error!(
						"Error updating render for bone {bone_kind:?}: {:?}",
						e
					);
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
	let (client, recv) = Client::new(CONNECT_STR.to_string(), subsys.clone())
		.wrap_err("Failed to start client")?;
	subsys.start("Overlay", |s| overlay(recv, s));
	client.join().await
}

fn is_overlay_topic(msg: Message<'_>) -> bool {
	if let Some(topic_id) = msg.topic_as_topic_id() {
		return matches!(topic_id.topic(), Some(TOPIC_DISPLAY_SETTINGS))
			&& matches!(topic_id.organization(), Some(TOPIC_ORG))
			&& matches!(topic_id.app_name(), Some(TOPIC_APP));
	} else if let Some(topic_handle) = msg.topic_as_topic_handle() {
		todo!("Check for topic handle")
	} else {
		false
	}
}
