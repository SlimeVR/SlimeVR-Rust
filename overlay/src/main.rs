mod color;
mod model;

pub use self::color::RGBA;

use crate::model::skeleton::SkeletonBuilder;
use crate::model::{BoneKind, Isometry};

use clap::Parser;
use eyre::{Result, WrapErr};
use git_version::git_version;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::encode::pattern::PatternEncoder;
use nalgebra::{Translation3, UnitQuaternion};
use ovr_overlay as ovr;
use solarxr::settings::DisplaySettings;
use solarxr::FeedUpdate;
use std::collections::HashSet;
use std::time::Duration;
use tokio::sync::watch;
use tokio_graceful_shutdown::{SubsystemHandle, Toplevel};

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};

const CONNECT_STR: &str = "ws://localhost:21110";
const GIT_VERSION: &str = git_version!();

#[derive(Parser, Debug)]
#[command(version = GIT_VERSION)]
struct Args {
	#[arg(long, default_value_t = false)]
	show_console: bool,
	#[arg(long, default_value_t = true)]
	show_log: bool,
}

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

#[cfg(target_os = "windows")]
fn hide_console_window() {
	use std::ptr;
	use winapi::um::wincon::GetConsoleWindow;
	use winapi::um::winuser::{ShowWindow, SW_HIDE};
	let window = unsafe { GetConsoleWindow() };
	if window != ptr::null_mut() {
		unsafe {
			ShowWindow(window, SW_HIDE);
		}
	}
}

fn init_log(show_log: bool) -> Result<()> {
	let logfile_name = "logfile.log";
	if std::path::Path::new(logfile_name).exists() {
		std::fs::remove_file(logfile_name)?;
	}

	let log_pattern = "{h({l})} - {d(%Y-%m-%d %H:%M:%S)} - {m}{n}";

	let window_size = 2;
	let fixed_window_roller = FixedWindowRoller::builder()
		.build("log_last_{}.log", window_size)
		.unwrap();
	let size_limit = 25 * 1024; // 25MB
	let size_trigger = SizeTrigger::new(size_limit);

	let compound_policy =
		CompoundPolicy::new(Box::new(size_trigger), Box::new(fixed_window_roller));

	let rolling_file_appender = RollingFileAppender::builder()
		.encoder(Box::new(PatternEncoder::new(log_pattern)))
		.build(logfile_name, Box::new(compound_policy))?;

	let mut config_builder = Config::builder();
	let mut root_builder = Root::builder();

	root_builder = root_builder.appender("logfile");
	config_builder = config_builder.appender(
		Appender::builder().build("logfile", Box::new(rolling_file_appender)),
	);

	if show_log {
		let stdout = ConsoleAppender::builder()
			.encoder(Box::new(PatternEncoder::new(log_pattern)))
			.build();
		root_builder = root_builder.appender("stdout");
		config_builder = config_builder
			.appender(Appender::builder().build("stdout", Box::new(stdout)));
	}

	let config = config_builder.build(root_builder.build(LevelFilter::Info))?;

	let _log = log4rs::init_config(config)?;

	Ok(())
}

#[tokio::main]
pub async fn main() -> Result<()> {
	if std::env::var("RUST_LOG").is_err() {
		std::env::set_var("RUST_LOG", "info");
	}

	let args = Args::parse();

	init_log(args.show_log)?;
	color_eyre::install()?;

	#[cfg(target_os = "windows")] {
		if !args.show_console {
			hide_console_window()
		}
	}

	log::info!("Overlay version: {GIT_VERSION}");

	Toplevel::new()
		.start("Networking", networking)
		.catch_signals()
		.handle_shutdown_requests(Duration::from_millis(1000))
		.await
		.wrap_err("system shutdown")
}

async fn overlay(
	mut recv: watch::Receiver<Option<FeedUpdate>>,
	display_settings: watch::Receiver<DisplaySettings>,
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
		loop {
			recv.changed()
				.await
				.wrap_err("Error while attempting to watch for feed update")?;
			let is_skeleton_visible = display_settings.borrow().is_visible;

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

			// Update all bones in datafeed
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
			}

			// Update rendering state
			for kind in BoneKind::iter() {
				skeleton.set_visibility(kind, !hidden_bones.contains(&kind));
				if let Err(e) = skeleton.update_render(kind, mngr) {
					log::error!("Error updating render for bone {kind:?}: {:?}", e);
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
	let (data_sender, data_reciever) = watch::channel(None);
	let (settings_sender, settings_receiver) =
		watch::channel(DisplaySettings::default());

	subsys.start("Overlay", |s| overlay(data_reciever, settings_receiver, s));

	let run_future = solarxr::run(CONNECT_STR.to_string(), |update| async {
		let ds = get_display_settings(&update).await;
		if let Some(ds) = ds {
			log::info!("Updating settings: {:?}", ds);
			settings_sender.send_replace(ds);
		}
		data_sender.send_replace(Some(update));
	});
	tokio::select! {
		_ = run_future => { unreachable!("This future never returns") },
		_ = subsys.on_shutdown_requested() => {
			log::debug!("networking shutdown requested");
			Ok(())
		}
	}
}

async fn get_display_settings<'a>(update: &FeedUpdate) -> Option<DisplaySettings> {
	let mut result = None;
	let Some(msgs) = update.0.table().pub_sub_msgs() else {
		return None;
	};
	for m in msgs {
		let Some(m) = m.u_as_message() else {
			continue;
		};
		log::debug!("Received pub-sub message with topic: {:?}", m.topic());

		if !solarxr::topic::is_overlay_topic(m) {
			continue;
		}

		// Check if they want to know current `DisplaySettings` (empty payload)
		if m.payload().is_none() {
			// TODO: Implement publishing the current settings
			continue;
		}

		let Some(kv) = m.payload_as_key_values() else {
			continue;
		};
		let ds = DisplaySettings::from_fb(kv);
		if ds.is_none() {
			log::warn!("Unable to parse `DisplaySettings` from flatbuffer");
			continue;
		};
		result = ds;
	}
	result
}
