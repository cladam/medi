use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub db_path: Option<PathBuf>,
    pub default_export_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        // Use the idiomatic data directory for the database.
        let default_db_path = dirs::data_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".medi")
            })
            .join("medi_db");

        // Use the Documents directory if available, otherwise fall back to a
        // directory inside the user's home.
        let default_export_dir = dirs::document_dir().or_else(|| {
            dirs::home_dir().map(|mut path| {
                path.push("medi_exports");
                path
            })
        });

        Config {
            db_path: Option::from(default_db_path),
            default_export_dir: Option::from(default_export_dir),
        }
    }
}

/// Loads the config from disk, creating a default one if it doesn't exist.
pub fn load() -> Result<Config, std::io::Error> {
    let config_dir = dirs::config_dir()
        .expect("Could not find config directory")
        .join("medi");

    // Create config directory if it doesn't exist.
    fs::create_dir_all(&config_dir)?;

    let config_path = config_dir.join("config.toml");

    // If the config file doesn't exist, create it with default values.
    if !config_path.exists() {
        let default_config = Config::default();
        let toml_string =
            toml::to_string_pretty(&default_config).expect("Could not serialize default config");
        fs::write(&config_path, toml_string)?;
    }

    // Read the config file from disk.
    let toml_content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&toml_content).expect("Could not deserialize config file");

    Ok(config)
}
