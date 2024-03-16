#![allow(irrefutable_let_patterns)]

mod config;

use std::error::Error;
use std::fs;
use std::io::Read;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use config::config_wizard;
use indicatif::{ProgressBar, ProgressStyle};

use hpm_isp::{
    hid,
    isp_command::{IspCommand, MemoryId},
};
use hpm_rt::XpiNorConfigurationOption;

const DEFAULT_CONFIG_FILE: &'static str = "hpm_isp.bin";

#[derive(Parser)]
#[clap(version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Command of xpi nor flash
    Flash {
        /// XPI<ID> to write or read (0-1)
        #[clap(parse(try_from_str = xpi_in_range))]
        id: MemoryId,
        #[clap(subcommand)]
        command: FlashCommands,
        /// Path of memory config file
        #[clap(short, long)]
        config: Option<PathBuf>,
    },
    /// Command of wizard to generate memory config file
    Wizard {
        /// Path of memory config file
        #[clap(short, long, default_value = DEFAULT_CONFIG_FILE)]
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum FlashCommands {
    /// Write file to xpi nor flash
    Write {
        /// Offset address to write
        #[clap(parse(try_from_str = parse_hex))]
        offset: u32,
        /// File to write
        file: PathBuf,
    },
    /// Read from xpi nor flash
    Read {
        /// Offset address to read
        #[clap(parse(try_from_str = parse_hex))]
        offset: u32,
        /// Bytes to read
        #[clap(parse(try_from_str = parse_hex))]
        size: u32,
        /// File to save
        file: PathBuf,
    },
}

fn parse_hex(s: &str) -> Result<u32, ParseIntError> {
    if s.starts_with("0x") {
        u32::from_str_radix(s.trim_start_matches("0x"), 16)
    } else {
        u32::from_str_radix(s, 10)
    }
}

fn xpi_in_range(s: &str) -> Result<MemoryId, String> {
    match s.parse() {
        Ok(0u32) => Ok(MemoryId::XPI0),
        Ok(1u32) => Ok(MemoryId::XPI1),
        Ok(_) => Err("ID must be 0 or 1".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if let Commands::Flash {
        id: memory_id,
        command: flash_command,
        config,
    } = cli.command
    {
        let device = hid::HpmDevice::open().or_else(|_| Err("can't open HPMicro usb device"))?;
        let mut memory_config_bin = Vec::new();
        let is_default = config.is_none();
        let config_path = config.unwrap_or(DEFAULT_CONFIG_FILE.into());

        match fs::File::open(config_path) {
            Ok(mut config_file) => {
                memory_config_bin.resize(12, 0);
                config_file.read_exact(memory_config_bin.as_mut_slice())?;
                if memory_config_bin[3] != 0xFC || memory_config_bin[2] != 0xF9 {
                    return Err("Invalid memory config file".into());
                }
            }
            // Use default config file and it isn't exists
            Err(_) if is_default => {
                let xpi_config = XpiNorConfigurationOption::new();
                xpi_config.write(&mut memory_config_bin)?;
            }
            Err(err) => {
                Err(err)?;
            }
        }

        // Config memory
        device.write_memory(MemoryId::ILM, 0x200, &memory_config_bin, |_, _| {})?;
        device.configure_memory(memory_id, MemoryId::ILM.base_address() + 0x200)?;

        match flash_command {
            FlashCommands::Write { offset, file } => {
                write_file(file, memory_id, offset, &device)?;
            }
            FlashCommands::Read { offset, size, file } => {
                read_file(file, memory_id, offset, size as usize, &device)?;
            }
        }
    } else if let Commands::Wizard { path } = cli.command {
        config_wizard(path)?;
    }

    Ok(())
}

fn write_file<D, P>(
    path: P,
    memory_id: MemoryId,
    offset: u32,
    device: &D,
) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
    D: IspCommand,
{
    // Write flash
    let pb = new_progress_bar(0);
    device.write_file(path, memory_id, offset, |w, l| {
        pb.set_length(l as u64);
        pb.set_position(w as u64);
    })?;
    pb.finish();
    Ok(())
}

fn read_file<D, P>(
    path: P,
    memory_id: MemoryId,
    offset: u32,
    length: usize,
    device: &D,
) -> Result<(), Box<dyn Error>>
where
    D: IspCommand,
    P: AsRef<Path>,
{
    // Read flash
    let pb = new_progress_bar(length as u64);
    device.read_file(path, memory_id, offset, length, |b, _| {
        pb.set_position(b as u64)
    })?;
    pb.finish();
    Ok(())
}

fn new_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));
    pb
}
