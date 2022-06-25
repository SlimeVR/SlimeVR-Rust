use clap::{IntoApp, Parser};
use eyre::Result;

const VERSION_STR: Option<&str> = option_env!("SLIMEVR_OVERLAY_VERSION");

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {}

fn main() -> Result<()> {
    pretty_env_logger::init();
    color_eyre::install()?;

    let args = Args::parse();

    slimevr_overlay::main()
}
