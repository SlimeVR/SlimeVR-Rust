mod errors;

use eyre::WrapErr;
use notify::Watcher;
use notify::{RecommendedWatcher, RecursiveMode};
use std::{path::Path, time::Duration};

const UART_PATH: &str = "/dev/ttyAMA0";
const USB_SERIAL_PATH: &str = "/dev/ttyUSB0";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    log::info!("Started program at {}", chrono::Utc::now());
    let cfg = notify::Config::default()
        .with_poll_interval(Duration::from_millis(1000))
        .with_compare_contents(false);

    // let (file_sender, file_receiver) = {
    //     use notify::{Event, Result};
    //     cfg_if::cfg_if! {
    //         if #[cfg(feature ="crossbeam")] {
    //             crossbeam_channel::bounded(1)
    //         } else {
    //             std::sync::mpsc::channel::<Result<Event>>()
    //         }
    //     }
    // };
    let (file_sender, mut file_receiver) = tokio::sync::mpsc::channel(1);

    let mut watcher = RecommendedWatcher::new(
        move |result| {
            let _ = file_sender.try_send(result);
        },
        cfg,
    )
    .wrap_err("Failed to create file watcher")?;
    watcher
        .watch(Path::new(UART_PATH), RecursiveMode::NonRecursive)
        .wrap_err("Failed to watch UART file path")?;
    watcher
        .watch(Path::new(USB_SERIAL_PATH), RecursiveMode::NonRecursive)
        .wrap_err("Failed to watch USB serial file path")?;

    loop {
        let file_result = file_receiver.recv().await;
    }
}
