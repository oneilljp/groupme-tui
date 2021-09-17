use dirs;

use std::{env, fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use toml;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub secret: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            secret: "".to_string(),
        }
    }
}

#[cfg(target_os = "linux")]
fn config_dir() -> Option<PathBuf> {
    let mut conf_dir = dirs::config_dir().unwrap();
    conf_dir.push("groupme-tui");
    Some(conf_dir)
}

#[cfg(target_os = "macos")]
fn config_dir() -> Option<PathBuf> {
    let mut conf_dir = dirs::config_dir().unwrap();
    conf_dir.push("Groupme-tui");
    Some(conf_dir)
}

#[cfg(target_os = "windows")]
fn config_dir() -> Option<PathBuf> {
    let mut conf_dir = dirs::config_dir().unwrap();
    conf_dir.push("Groupme-tui");
    Some(conf_dir)
}

// Returns Config Struct Based on config file, creating one when necessary
pub fn get_configs() -> Option<Config> {
    let mut conf_dir = match env::var("GMTUI_CONFIG") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => config_dir().unwrap(),
    };

    // Create config directory if necessary
    if !conf_dir.is_dir() {
        fs::create_dir_all(&conf_dir).unwrap();
    }

    // Potentially look into creating default config file for base settings,
    // Then overwriting the defaults with the custom config, assuming that
    // more configuration is implemented outside of just the secret
    conf_dir.push("config.toml");
    let config_file = conf_dir;

    if !config_file.is_file() {
        let mut secret = String::new();
        println!("Enter GroupMe Access Token, which can be obtained here: https://dev.groupme.com/applications");
        io::stdin()
            .read_line(&mut secret)
            .expect("Failed to read Access Token");
        secret = secret.trim().to_string();
        let config = Config { secret };
        fs::write(&config_file, toml::to_string(&config).unwrap()).unwrap();
    }

    let contents = fs::read_to_string(config_file).unwrap();

    let config: Config = toml::from_str(&contents).unwrap();

    Some(config)
}
