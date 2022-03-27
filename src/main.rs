use eyre::Result;

fn main() -> Result<()> {
    pretty_env_logger::init();
    color_eyre::install()?;

    slimevr_overlay::main()
}
