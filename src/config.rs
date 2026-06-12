use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub gif_path: Option<String>,
    pub rotate_clockwise: Option<bool>,
    pub target_size: Option<u32>,
}

impl Config {
    pub fn load(cli_config: &Option<String>) -> Self {
        let mut config = Config::default();

        let cfg_path = if let Some(cp) = cli_config {
            Some(PathBuf::from(cp))
        } else {
            directories::ProjectDirs::from("", "", "gif_walker")
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
            } else if let Some(parent) = cp.parent() {
                let _ = fs::create_dir_all(parent);
                let default_toml = r#"
# Default configuration for gif_walker
# gif_path = "./mushroom.gif"
# rotate_clockwise = false
# target_size = 40
"#;
                let _ = fs::write(&cp, default_toml.trim_start());
            }
        }
        config
    }
}
