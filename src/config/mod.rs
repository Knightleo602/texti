use crate::config::app_config::AppConfig;
use crate::config::keybindings::Keybindings;
use color_eyre::Result;
use config::File;
use config::FileFormat::Yaml;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;

mod app_config;
pub(crate) mod effects_config;
pub(crate) mod keybindings;

const CONFIG_FILE_NAME: &str = "config.yaml";
const CONFIG: &str = include_str!("../../.config/config.yaml");

lazy_static! {
    pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
    static ref DATA_FOLDER: Option<PathBuf> = env::var(format!("{}_DATA", PROJECT_NAME.clone()))
        .ok()
        .map(PathBuf::from);
    static ref CONFIG_FOLDER: Option<PathBuf> =
        env::var(format!("{}_CONFIG", PROJECT_NAME.clone()))
            .ok()
            .map(PathBuf::from);
}

#[derive(Clone, Default, Debug, Deserialize)]
pub struct Config {
    #[serde(default, flatten)]
    pub config: AppConfig,
    #[serde(default)]
    pub keybindings: Keybindings,
}

impl Config {
    pub fn new() -> Result<Config> {
        let default_config = serde_yaml::from_str::<Config>(CONFIG)?;
        let config_dir = get_config_dir();
        let file = config_dir.join(CONFIG_FILE_NAME);
        let source = File::from(file.clone()).format(Yaml).required(false);
        let config = config::Config::builder()
            .set_default("config_dir", config_dir.to_str().unwrap())?
            .add_source(source);
        let mut config: Config = config.build()?.try_deserialize()?;
        for (app_component, default_bindings) in default_config.keybindings.iter() {
            let user_bindings = config.keybindings.entry(app_component.clone()).or_default();
            for (key, cmd) in default_bindings.iter() {
                user_bindings.entry(*key).or_insert_with(|| cmd.clone());
            }
        }
        Ok(config)
    }
}

pub fn get_config_dir() -> PathBuf {
    if let Some(s) = CONFIG_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".config")
    }
}

pub fn get_config_file_dir() -> PathBuf {
    get_config_dir().join("config.yml")
}

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("io", "github.knightleo", env!("CARGO_PKG_NAME"))
}
