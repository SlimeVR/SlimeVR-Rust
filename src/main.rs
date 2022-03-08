use std::time::Duration;

use eyre::{Result, WrapErr};
use openxr as xr;

fn main() -> Result<()> {
    let entry = if let Ok(entry) = xr::Entry::load() {
        entry
    } else {
        xr::Entry::linked()
    };

    let extensions = entry.enumerate_extensions().unwrap();
    println!("supported extensions: {:#?}", extensions);
    let layers = entry.enumerate_layers().unwrap();
    println!("supported layers: {:?}", layers);

    let instance = entry
        .create_instance(
            &xr::ApplicationInfo {
                application_name: "SlimeVR Overlay",
                ..Default::default()
            },
            &xr::ExtensionSet::default(),
            &[],
        )
        .wrap_err("Failed to create OpenXR instance")?;
    let instance_props = instance.properties().unwrap();
    println!(
        "loaded instance: {} v{}",
        instance_props.runtime_name, instance_props.runtime_version
    );

    let system = instance
        .system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
        .wrap_err("Could not create OpenXR System")?;
    let properties = instance
        .system_properties(system)
        .wrap_err("Could not get system properties")?;
    println!("HMD System properties: {properties:?}");

    // We cannot create the session to read the input actions without initializing
    // a graphics context
    // let session = unsafe { instance.create_session(system, info) };

    let left_pose_path = instance
        .string_to_path("/user/hand/left/input/grip/pose")
        .wrap_err("Failed to get left hand pose path")?;
    let right_pose_path = instance
        .string_to_path("/user/hand/right/input/grip/pose")
        .wrap_err("Failed to get right hand pose path")?;

    loop {
        std::thread::sleep(Duration::from_millis(1));
    }
}
