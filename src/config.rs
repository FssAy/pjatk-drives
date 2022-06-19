use crate::logging::*;
use config::{ConfigError, FileFormat};
use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use std::str::FromStr;

/// Main server's config
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub bind: SocketAddr,
    pub host: String,
    pub api_version: u8,
    pub ftp_host: SocketAddr,
    #[deprecated]
    pub ftp_domain: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind: SocketAddr::from_str("127.0.0.1:6655").unwrap(),
            host: "http://127.0.0.1:6655".to_string(),
            api_version: 1,
            ftp_host: SocketAddr::from_str("91.230.222.36:22").unwrap(),
            ftp_domain: "sftp.pjwstk.edu.pl".to_string(),
        }
    }
}

impl Config {
    /// Tries to read a config file, or creates it with the default values if it doesn't exist
    pub fn new() -> anyhow::Result<Self> {
        let cfg = match config::Config::builder()
            .add_source(config::File::new("config", FileFormat::Json))
            .build()
        {
            Ok(config) => config.try_deserialize::<Config>()?,

            Err(error) => match &error {
                ConfigError::Foreign(_) => {
                    warn!("No valid config file was found, creating a new one.");

                    let config = Config::default();
                    let mut config_file = File::create("config.json")?;

                    config_file
                        .write(serde_json::to_string(&config).unwrap().as_bytes())
                        .ok();
                    drop(config_file);

                    config
                }
                _ => {
                    return Err(error.into());
                }
            },
        };

        Ok(cfg)
    }
}
