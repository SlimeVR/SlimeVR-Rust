use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};
use url::Url;

#[derive(Deserialize, Debug)]
pub struct ComponentInfo {
    url: Url,
    install_path: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct Components {
    components: HashMap<String, ComponentInfo>,
}
