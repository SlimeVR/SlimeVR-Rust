// mod parsing;

use clap::Parser;
use eyre::{Result, WrapErr};
use lazy_static::lazy_static;
use reqwest::Url;

lazy_static! {
    static ref VERSIONING_URL: Url = Url::parse(
        "https://github.com/SlimeVR/SlimeVR-Overlay/releases/download/autoupdater-latest/versioning.json"
    ).unwrap();
}

#[derive(Parser)]
struct Args {
    #[clap(long, default_value_t = VERSIONING_URL.clone())]
    url: Url,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let body = reqwest::get(args.url.clone())
        .await
        .wrap_err_with(|| {
            format!(
                "Failed to download `versioning.json` from github! URL: {}",
                &args.url
            )
        })?
        .text()
        .await
        .wrap_err("Failed to decode response body")?;

    // let components: Components = serde_yaml::from_str(&body).wrap_err_with(|| {
    // format!("Could not deserialize YAML, response was:\n{body}")
    // })?;
    // println!("components: {components:?}");
    Ok(())
}
