#![doc = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/",
    env!("CARGO_PKG_README")
))]

pub mod hid;
pub mod isp_command;
pub mod memory_config;
