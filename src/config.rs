use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

const APP_QUALIFIER: &str = "com";
const APP_ORGANIZATION: &str = "mushy";
const APP_NAME: &str = "mushy";

pub const DEFAULT_TARGET_SIZE: u32 = 40;
pub const DEFAULT_SPEED: f32 = 1.0;
pub const DEFAULT_ROTATE_CLOCKWISE: bool = false;

fn default_target_size() -> u32 { DEFAULT_TARGET_SIZE }
fn default_speed() -> f32 { DEFAULT_SPEED }
fn default_rotate_clockwise() -> bool { DEFAULT_ROTATE_CLOCKWISE }

#[derive(Deserialize, Debug)]
pub struct Config {
    pub gif_path: Option<String>,
    #[serde(default = "default_rotate_clockwise")]
    pub rotate_clockwise: bool,
    #[serde(default = "default_target_size")]
    pub target_size: u32,
    #[serde(default = "default_speed")]
    pub speed: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gif_path: None,
            rotate_clockwise: DEFAULT_ROTATE_CLOCKWISE,
            target_size: DEFAULT_TARGET_SIZE,
            speed: DEFAULT_SPEED,
        }
    }
}

impl Config {
    pub fn load(cli_config: &Option<String>) -> Self {
        let mut config = Config::default();

        let cfg_path = if let Some(cp) = cli_config {
            Some(PathBuf::from(cp))
        } else {
            directories::ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
                .map(|proj_dirs| proj_dirs.config_dir().join("config.toml"))
        };

        if let Some(cp) = cfg_path {
            if cp.exists() {
                if let Ok(contents) = fs::read_to_string(&cp) {
                    if let Ok(parsed) = toml::from_str(&contents) {
                        config = parsed;
                    } else {
                        eprintln!("Warning: Failed to parse config file at {:?}", cp);
                    }
                }
            }
        }
        config
    }
}
