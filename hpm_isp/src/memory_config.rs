use hpm_rt::{
    FlashType as HpmFlashType, PinGroup as HpmPinGroup, PortConnection as HpmPortConnection,
    QuadIOEnableSequence as HpmQuadIOEnableSequence, XpiNorConfigurationOption,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const MEMORY_CONFIG_LEN: usize = 12;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to parse memory config TOML")]
    Parse(#[from] toml::de::Error),
    #[error("failed to serialize memory config TOML")]
    Serialize(#[from] toml::ser::Error),
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlashType {
    #[default]
    SfdpSdr,
    SfdpDdr,
    #[serde(rename = "read_1_4_4")]
    Read144,
    #[serde(rename = "read_1_2_2")]
    Read122,
    #[serde(rename = "hyperbus_1v8")]
    HyperBus1v8,
    #[serde(rename = "hyperbus_3v3")]
    HyperBus3v3,
    OctaBusDdr,
    XccelaDdr,
    EcoXipDdr,
}

impl From<FlashType> for HpmFlashType {
    fn from(flash_type: FlashType) -> Self {
        match flash_type {
            FlashType::SfdpSdr => Self::SfdpSdr,
            FlashType::SfdpDdr => Self::SfdpDdr,
            FlashType::Read144 => Self::Read144,
            FlashType::Read122 => Self::Read122,
            FlashType::HyperBus1v8 => Self::HyperBus1v8,
            FlashType::HyperBus3v3 => Self::HyperBus3v3,
            FlashType::OctaBusDdr => Self::OctaBusDdr,
            FlashType::XccelaDdr => Self::XccelaDdr,
            FlashType::EcoXipDdr => Self::EcoXipDdr,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PinGroup {
    #[default]
    Group1,
    Group2,
}

impl From<PinGroup> for HpmPinGroup {
    fn from(group: PinGroup) -> Self {
        match group {
            PinGroup::Group1 => Self::Group1,
            PinGroup::Group2 => Self::Group2,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortConnection {
    #[default]
    PortACs0,
    PortBCs0,
    PortACs0PortBCs0,
    PortACs0PortACs1,
    PortBCs0PortBCs1,
}

impl From<PortConnection> for HpmPortConnection {
    fn from(port: PortConnection) -> Self {
        match port {
            PortConnection::PortACs0 => Self::PortACs0,
            PortConnection::PortBCs0 => Self::PortBCs0,
            PortConnection::PortACs0PortBCs0 => Self::PortACs0PortBCs0,
            PortConnection::PortACs0PortACs1 => Self::PortACs0PortACs1,
            PortConnection::PortBCs0PortBCs1 => Self::PortBCs0PortBCs1,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QuadIOEnableSequence {
    #[default]
    None,
    Status1Bit6,
    Status2Bit1,
    Status2Bit7,
    #[serde(rename = "status2_bit1_programmed_by_0x31")]
    Status2Bit1ProgrammedBy0x31,
}

impl From<QuadIOEnableSequence> for HpmQuadIOEnableSequence {
    fn from(sequence: QuadIOEnableSequence) -> Self {
        match sequence {
            QuadIOEnableSequence::None => Self::None,
            QuadIOEnableSequence::Status1Bit6 => Self::Status1Bit6,
            QuadIOEnableSequence::Status2Bit1 => Self::Status2Bit1,
            QuadIOEnableSequence::Status2Bit7 => Self::Status2Bit7,
            QuadIOEnableSequence::Status2Bit1ProgrammedBy0x31 => Self::Status2Bit1ProgrammedBy0x31,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MemoryConfig {
    flash_type: FlashType,
    port_connection: PortConnection,
    pin_group: PinGroup,
    quad_io_enable_sequence: QuadIOEnableSequence,
}

impl MemoryConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_toml_str(config: &str) -> Result<Self, ConfigError> {
        Ok(toml::from_str(config)?)
    }

    pub fn to_toml_string(&self) -> Result<String, ConfigError> {
        Ok(toml::to_string_pretty(self)?)
    }

    pub fn flash_type(mut self, flash_type: FlashType) -> Self {
        self.flash_type = flash_type;
        self
    }

    pub fn port_connection(mut self, port_connection: PortConnection) -> Self {
        self.port_connection = port_connection;
        self
    }

    pub fn pin_group(mut self, pin_group: PinGroup) -> Self {
        self.pin_group = pin_group;
        self
    }

    pub fn quad_io_enable_sequence(
        mut self,
        quad_io_enable_sequence: QuadIOEnableSequence,
    ) -> Self {
        self.quad_io_enable_sequence = quad_io_enable_sequence;
        self
    }

    pub fn to_bootrom_config(&self) -> Vec<u8> {
        let mut memory_config_bin = Vec::with_capacity(MEMORY_CONFIG_LEN);
        self.xpi_nor_configuration_option()
            .write(&mut memory_config_bin)
            .unwrap();
        memory_config_bin
    }

    fn xpi_nor_configuration_option(&self) -> XpiNorConfigurationOption {
        XpiNorConfigurationOption::new()
            .flash_type(self.flash_type.into())
            .connect_port(self.port_connection.into())
            .pin_group(self.pin_group.into())
            .quad_io_enable_sequence(self.quad_io_enable_sequence.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_toml_config_values() {
        let config = MemoryConfig::from_toml_str(
            r#"
flash_type = "read_1_4_4"
port_connection = "port_b_cs0"
pin_group = "group2"
quad_io_enable_sequence = "status2_bit1_programmed_by_0x31"
"#,
        )
        .unwrap();

        assert_eq!(config.flash_type, FlashType::Read144);
        assert_eq!(config.port_connection, PortConnection::PortBCs0);
        assert_eq!(config.pin_group, PinGroup::Group2);
        assert_eq!(
            config.quad_io_enable_sequence,
            QuadIOEnableSequence::Status2Bit1ProgrammedBy0x31
        );
    }

    #[test]
    fn rejects_unknown_toml_fields() {
        assert!(MemoryConfig::from_toml_str("unknown = true").is_err());
    }

    #[test]
    fn rejects_toml_section_header() {
        assert!(MemoryConfig::from_toml_str("[memory_config]\nflash_type = \"sfdp_sdr\"").is_err());
    }

    #[test]
    fn serializes_toml_config_fields() {
        let config = MemoryConfig::default().to_toml_string().unwrap();

        assert!(!config.contains("[memory_config]"));
        assert!(config.contains("flash_type = \"sfdp_sdr\""));
    }

    #[test]
    fn writes_bootrom_config_magic() {
        let config = MemoryConfig::default().to_bootrom_config();

        assert_eq!(config.len(), MEMORY_CONFIG_LEN);
        assert_eq!(config[2], 0xF9);
        assert_eq!(config[3], 0xFC);
    }
}
