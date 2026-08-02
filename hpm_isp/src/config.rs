use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use hpm_isp::memory_config::MemoryConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    memory_config: MemoryConfig,
}

impl Config {
    pub(crate) fn new(memory_config: MemoryConfig) -> Self {
        Self { memory_config }
    }

    fn from_file<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        println!("Reading memory config from: {}", path.as_ref().display());

        let config = fs::read_to_string(path)?;
        Ok(Self::from_toml_str(&config)?)
    }

    pub(crate) fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    fn into_memory_config(self) -> MemoryConfig {
        self.memory_config
    }

    fn from_toml_str(config: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(config)
    }
}

pub(crate) fn read_memory_config_or_default(
    config: Option<PathBuf>,
    default_config_file: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    if let Some(config_path) = config {
        return read_memory_config(config_path);
    }

    if Path::new(default_config_file).exists() {
        return read_memory_config(default_config_file);
    }

    Ok(MemoryConfig::default().to_bootrom_config())
}

fn read_memory_config<P>(path: P) -> Result<Vec<u8>, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    Ok(Config::from_file(path)?
        .into_memory_config()
        .to_bootrom_config())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_memory_config_section() {
        let config = Config::from_toml_str(
            r#"
[memory_config]
flash_type = "read_1_4_4"
port_connection = "port_b_cs0"
pin_group = "group2"
quad_io_enable_sequence = "status2_bit1_programmed_by_0x31"
"#,
        )
        .unwrap();

        assert_eq!(config.into_memory_config().to_bootrom_config().len(), 12);
    }

    #[test]
    fn serializes_memory_config_section() {
        let config = Config::new(MemoryConfig::default())
            .to_toml_string()
            .unwrap();

        assert!(config.contains("[memory_config]"));
        assert!(config.contains("flash_type = \"sfdp_sdr\""));
    }
}
