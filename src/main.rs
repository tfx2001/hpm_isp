#![allow(irrefutable_let_patterns)]

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::num::ParseIntError;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};

use hpm_isp::{hid, isp_command::{IspCommand, MemoryId}};

const MEMORY_CONFIG: [u8; 8] = [0x01, 0x00, 0xF9, 0xFC, 0x07, 0x00, 0x00, 0x00];

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Command of xpi nor flash
    Flash {
        /// XPI<ID> to write or read
        #[clap(parse(try_from_str = xpi_in_range))]
        id: MemoryId,
        #[clap(subcommand)]
        command: FlashCommands,
    }
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
        /// How many bytes to read
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
        Err(e) => Err(e.to_string())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let device = hid::HpmDevice::open().or_else(|_| Err("can't open HPMicro usb device"))?;


    if let Commands::Flash { id, command: flash_command } = cli.command {
        match flash_command {
            FlashCommands::Write { offset, file } => {
                write_file(file, id, offset, &device)?;
            }
            FlashCommands::Read { offset, size, file } => {
                read_file(file, id, offset, size as usize, &device)?;
            }
        }
    }

    Ok(())
}

fn write_file<D, P>(path: P, memory_id: MemoryId, offset: u32, device: &D) -> Result<(), Box<dyn Error>>
    where P: AsRef<Path>,
          D: IspCommand
{
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();

    // Configure flash
    device.write_memory(MemoryId::ILM,
                        0x200,
                        &MEMORY_CONFIG,
                        |_, _| {})?;
    device.configure_memory(memory_id, 0x0000_0200)?;
    // Write flash
    file.read_to_end(&mut buffer).unwrap();
    let pb = new_progress_bar(buffer.len() as u64);
    device.write_memory(memory_id, offset, &buffer,
                        |w, _| pb.set_position(w as u64))?;
    pb.finish();
    Ok(())
}

fn read_file<D, P>(path: P, memory_id: MemoryId, offset: u32, length: usize, device: &D) -> Result<(), Box<dyn Error>>
    where D: IspCommand,
          P: AsRef<Path>
{
    let mut buffer = Vec::new();

    // Configure flash
    device.write_memory(MemoryId::ILM,
                        0x200,
                        &MEMORY_CONFIG,
                        |_, _| {})?;
    device.configure_memory(memory_id, 0x0000_0200)?;
    // Read flash
    let pb = new_progress_bar(length as u64);
    buffer.resize(length, 0);
    device.read_memory(memory_id, offset, &mut buffer,
                       |b, _| pb.set_position(b as u64))?;
    pb.finish();

    let mut file = File::create(path)?;
    file.write_all(&buffer).unwrap();
    Ok(())
}

fn new_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .progress_chars("#>-"));
    pb
}
