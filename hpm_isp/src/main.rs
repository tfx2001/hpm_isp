mod config;
mod wizard;

use std::error::Error;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use config::read_memory_config_or_default;
use indicatif::{ProgressBar, ProgressStyle};
use wizard::config_wizard;

use hpm_isp::{
    hid,
    isp_command::{IspCommand, MemoryId},
};

const DEFAULT_CONFIG_FILE: &str = "hpm_isp.toml";

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
        s.parse::<u32>()
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

    match cli.command {
        Commands::Flash {
            id: memory_id,
            command: flash_command,
            config,
        } => {
            let device = hid::HpmDevice::open().map_err(|_| "can't open HPMicro usb device")?;
            let memory_config_bin = read_memory_config_or_default(config, DEFAULT_CONFIG_FILE)?;

            println!("Found chip: {}", device.family());

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
        }
        Commands::Wizard { path } => {
            config_wizard(path)?;
        }
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
