mod parsing;

use autoupdater::{
	component::{
		AvailableComponentsFile, Component, Components, InstalledComponentsFile,
	},
	util::SelectableHashMap,
};
use clap::Parser;
use color_eyre::eyre;
use eyre::Result;
use lazy_static::lazy_static;
use reqwest::Url;
use std::path::PathBuf;

lazy_static! {
	static ref VERSIONING_URL: Url = Url::parse(
		"https://github.com/SlimeVR/SlimeVR-Overlay/releases/download/autoupdater-latest/versioning.json"
	).unwrap();
}

#[derive(Parser)]
struct Args {
	/// The url to fetch the versioning.yaml from.
	#[clap(long, default_value_t = VERSIONING_URL.clone())]
	url: Url,
	#[clap(long)]
	/// The path to a versioning.yaml file, to use instead of the URL.
	path: Option<PathBuf>,
}

/// Helper macro to unpack Option<T> and continue with a helpful error if None.
macro_rules! try_get {
	($arg:expr) => {
		if let Some(v) = $arg {
			v
		} else {
			println!("Component not supported on this platform. Skipping.");
			continue;
		}
	};
}

#[tokio::main]
async fn main() -> Result<()> {
	let args = Args::parse();
	color_eyre::install()?;

	let installed_components =
		InstalledComponentsFile::load("installed_components.yml".into())?;
	installed_components.save("installed_components.yml".into())?;

	let all_installable_components =
		AvailableComponentsFile::load("components.yml".into())?;
	let components: Components = all_installable_components.into();

	println!("Compatible components:");
	for (_, c) in components.compatible() {
		println!("- {}", c);
	}

	println!("Incompatible components:");
	for (_, c) in components.incompatible() {
		println!("- {}", c);
	}
	println!("");

	let mut thing = components.into_selectable_hashmap();
	// dbg!(&thing);

	thing.enable(&"slimevr_gui".to_string());

	for (_, c) in thing.selected_items_iter() {
		println!("{}", c);
	}

	// let versioning = if let Some(p) = args.path {
	// 	std::fs::read_to_string(p)
	// 		.wrap_err("Failed to read versioning.yaml from file")?
	// } else {
	// 	reqwest::get(args.url.clone())
	// 		.await
	// 		.wrap_err_with(move || {
	// 			format!(
	// 				"Failed to download `versioning.json` from URL: {}",
	// 				&args.url
	// 			)
	// 		})?
	// 		.text()
	// 		.await
	// 		.wrap_err("Failed to decode response body")?
	// };

	// // Parse/deserialize yaml
	// let components: Components =
	// 	serde_yaml::from_str(&versioning).wrap_err_with(|| {
	// 		format!("Could not deserialize YAML, whose contents was:\n{versioning}")
	// 	})?;

	// // We will delete the temporary dir upon drop.
	// let tmp_dir = tempdir().wrap_err("Failed to create temporary directory")?;

	// // Download each component, storing the async tasks in `download_tasks`
	// let mut download_tasks = Vec::new();
	// for (comp_name, comp_info) in components.0.into_iter() {
	// 	println!("Downloading component: {comp_name}...");
	// 	let url = try_get!(comp_info.download_url.get_owned());
	// 	let install_path = try_get!(comp_info.install_dir.get_owned());

	// 	let mut response = reqwest::get(url.clone()).await.wrap_err_with(|| {
	// 		format!("Failed to download `{comp_name}` from URL: {url}")
	// 	})?;

	// 	// TODO: This should use snake_case names since that is what the components in the
	// 	// file use.
	// 	let filename = url
	// 		.path_segments()
	// 		.map_or(comp_name.to_string(), |v| v.last().unwrap().to_string());
	// 	let download_path = tmp_dir.path().join(filename);
	// 	let mut download_file = File::create(&download_path)
	// 		.await
	// 		.wrap_err("Failed to create temporary file")?;

	// 	// We spawn a task so that the file can be downloaded concurrently
	// 	let task: JoinHandle<Result<_>> = tokio::spawn(async move {
	// 		while let Some(mut b) = response
	// 			.chunk()
	// 			.await
	// 			.wrap_err("error while slurping chunks")?
	// 		{
	// 			download_file
	// 				.write_all_buf(&mut b)
	// 				.await
	// 				.wrap_err("Error while writing to file")?
	// 		}
	// 		Ok((download_file, download_path, install_path.to_path()?))
	// 	});
	// 	download_tasks.push(task);
	// }

	// // Check that all components downloaded successfully
	// let downloads: Result<Vec<_>> = join_all(download_tasks)
	// 	.await
	// 	.into_iter()
	// 	.map(|t| {
	// 		t.wrap_err("couldn't join task")?
	// 			.wrap_err("failed download of component")
	// 	})
	// 	.collect();
	// let downloads = downloads?;

	// // Check that all files are writeable *before* attempting to move.
	// join_all(downloads.iter().map(|(_, _, install_path)| async move {
	// 	OpenOptions::new()
	// 		.create(true)
	// 		.write(true)
	// 		.open(install_path)
	// 		.await
	// 		.wrap_err_with(|| format!("{install_path:?} was not writeable"))
	// }))
	// .await
	// .into_iter()
	// .collect::<Result<Vec<_>>>()?; // returns if errors

	// // Move all the components to overwrite the old ones.
	// // This is suceptible to race conditions after the writable check, but its fine 😅
	// for (download_file, download_path, ref install_path) in downloads {
	// 	drop(download_file);
	// 	// Truncates if the file already exists.
	// 	tokio::fs::rename(download_path, install_path)
	// 		.await
	// 		.wrap_err_with(|| {
	// 			format!("Failed to move temporary file to {install_path:?}")
	// 		})?;
	// }

	// println!("Press enter to quit...");
	// std::io::stdin().read_line(&mut String::new()).unwrap();

	Ok(())
}
