#![allow(unused)]

use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::{cmp, error, fmt, io, mem};

use zerocopy::{AsBytes, FromBytes, FromZeroes};

#[derive(AsBytes, FromZeroes, FromBytes)]
#[repr(C, packed)]
pub struct Packet {
    cmd: u8,
    arg_num: u8,
    cmd_type: u8,
    reserved: u8,
    payload: [u8; 508],
}

#[repr(u8)]
enum Commands {
    /// Query runtime environment
    QueryRuntimeEnv = 0x01,
    /// Configure runtime environment
    ConfigureRuntimeEnv = 0x02,
    /// Configure memory
    ConfigureMemory = 0x03,
    /// Write memory
    WriteMemory = 0x04,
    /// Read memory
    ReadMemory = 0x05,
}

#[repr(u8)]
enum CommandType {
    CommandData = 0x00,
    DataOnly = 0x01,
    ResponseOnly = 0x02,
}

#[derive(AsBytes)]
#[repr(C, packed)]
struct QueryRuntimeEnvironment {
    id: RuntimeEnvironment,
}

impl QueryRuntimeEnvironment {
    fn new(id: RuntimeEnvironment) -> Self {
        Self { id }
    }
}

impl From<QueryRuntimeEnvironment> for Packet {
    fn from(query_rte: QueryRuntimeEnvironment) -> Self {
        let mut payload: [u8; 508] = [0; 508];
        payload[..mem::size_of::<QueryRuntimeEnvironment>()].copy_from_slice(query_rte.as_bytes());
        Packet {
            cmd: Commands::QueryRuntimeEnv as u8,
            arg_num: 1,
            cmd_type: CommandType::CommandData as u8,
            reserved: 0,
            payload,
        }
    }
}

#[derive(AsBytes)]
#[repr(u32)]
pub enum RuntimeEnvironment {
    RomParameter = 0x00,
    ActivePeripheralInfo = 0x01,
    LastBootStatus = 0x03,
    MemoryAttribute = 0x04,
}

struct RomParameter {
    /// TODO: 完善具体字段
    none: u8,
}

#[derive(AsBytes)]
#[repr(C, packed)]
struct ConfigureMemory {
    memory_id: u32,
    cfg_addr: u32,
}

impl ConfigureMemory {
    fn new(cfg_addr: u32, memory_id: MemoryId) -> Self {
        ConfigureMemory {
            memory_id: memory_id as u32,
            cfg_addr,
        }
    }
}

impl From<ConfigureMemory> for Packet {
    fn from(configure_memory: ConfigureMemory) -> Self {
        let mut payload: [u8; 508] = [0; 508];
        payload[..mem::size_of::<ConfigureMemory>()].copy_from_slice(configure_memory.as_bytes());
        Packet {
            cmd: Commands::ConfigureMemory as u8,
            arg_num: 2,
            cmd_type: CommandType::ResponseOnly as u8,
            reserved: 0,
            payload,
        }
    }
}

#[derive(AsBytes)]
#[repr(C, packed)]
struct WriteMemory {
    start: u32,
    length: u32,
    memory_id: u32,
}

impl WriteMemory {
    fn new(start: u32, length: u32, memory_id: MemoryId) -> Self {
        WriteMemory {
            start,
            length,
            memory_id: memory_id as u32,
        }
    }
}

impl From<WriteMemory> for Packet {
    fn from(write_memory: WriteMemory) -> Self {
        let mut payload: [u8; 508] = [0; 508];
        payload[..mem::size_of::<WriteMemory>()].copy_from_slice(write_memory.as_bytes());
        Packet {
            cmd: Commands::WriteMemory as u8,
            arg_num: 3,
            cmd_type: CommandType::CommandData as u8,
            reserved: 0,
            payload,
        }
    }
}

#[derive(AsBytes)]
#[repr(C, packed)]
struct ReadMemory {
    start: u32,
    length: u32,
    memory_id: u32,
}

impl ReadMemory {
    fn new(start: u32, length: u32, memory_id: MemoryId) -> Self {
        ReadMemory {
            start,
            length,
            memory_id: memory_id as u32,
        }
    }
}

impl From<ReadMemory> for Packet {
    fn from(write_memory: ReadMemory) -> Self {
        let mut payload: [u8; 508] = [0; 508];
        payload[..mem::size_of::<ReadMemory>()].copy_from_slice(write_memory.as_bytes());
        Packet {
            cmd: Commands::ReadMemory as u8,
            arg_num: 3,
            cmd_type: CommandType::CommandData as u8,
            reserved: 0,
            payload,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum MemoryId {
    ILM = 0x00,
    DLM = 0x01,
    XRAM = 0x02,
    XPI0 = 0x10000,
    XPI1 = 0x10001,
}

impl MemoryId {
    pub fn base_address(&self) -> u32 {
        match self {
            MemoryId::ILM => 0x0000_0000,
            MemoryId::DLM => 0x0008_0000,
            MemoryId::XRAM => 0x0108_0000,
            MemoryId::XPI0 => 0x8000_0000,
            MemoryId::XPI1 => 0x9000_0000,
        }
    }
}

#[derive(FromZeroes, FromBytes)]
#[repr(C, packed)]
struct GenericCommandResponse {
    status: u32,
}

impl From<GenericCommandResponse> for Result<(), Error> {
    fn from(resp: GenericCommandResponse) -> Self {
        if resp.status == 0 {
            Ok(())
        } else {
            Err(Error::Other(resp.status))
        }
    }
}

pub trait Interface {
    fn write(&self, packet: &Packet, length: u16) -> Result<(), Error>;
    fn read(&self, packet: &mut Packet) -> Result<u16, Error>;
}

fn write_from_reader<D, R, F>(
    device: &D,
    memory_id: MemoryId,
    offset: u32,
    total_length: usize,
    mut reader: R,
    update_progress: F,
) -> Result<(), Error>
where
    D: Interface + ?Sized,
    R: Read,
    F: Fn(usize, usize),
{
    let command_length = mem::size_of::<WriteMemory>();
    let mut packet: Packet = WriteMemory::new(
        offset + memory_id.base_address(),
        total_length as u32,
        memory_id,
    )
    .into();

    if total_length == 0 {
        device.write(&packet, command_length as u16)?;
        device.read(&mut packet)?;
        let resp = GenericCommandResponse::read_from_prefix(&packet.payload[..]).unwrap();
        return resp.into();
    }

    let mut bytes_written = 0;
    let mut max_length = packet.payload.len() - command_length;
    let mut payload_offset = command_length;

    while bytes_written < total_length {
        let write_length = cmp::min(max_length, total_length - bytes_written);
        reader.read_exact(&mut packet.payload[payload_offset..payload_offset + write_length])?;
        device.write(&packet, (payload_offset + write_length) as u16)?;
        bytes_written += write_length;
        update_progress(bytes_written, total_length);

        packet.arg_num = 0;
        packet.cmd_type = CommandType::DataOnly as u8;
        payload_offset = 0;
        max_length = packet.payload.len();
    }

    device.read(&mut packet)?;
    let resp = GenericCommandResponse::read_from_prefix(&packet.payload[..]).unwrap();
    resp.into()
}

fn read_to_writer<D, W, F>(
    device: &D,
    memory_id: MemoryId,
    offset: u32,
    total_length: usize,
    mut writer: W,
    update_progress: F,
) -> Result<(), Error>
where
    D: Interface + ?Sized,
    W: Write,
    F: Fn(usize, usize),
{
    let mut packet: Packet = ReadMemory::new(
        offset + memory_id.base_address(),
        total_length as u32,
        memory_id,
    )
    .into();
    let mut bytes_read = 0;

    device.write(&packet, mem::size_of::<ReadMemory>() as u16)?;
    device.read(&mut packet)?;
    let resp = GenericCommandResponse::read_from_prefix(&packet.payload[..]).unwrap();
    if resp.status != 0 {
        return resp.into();
    }

    while bytes_read < total_length {
        let length = device.read(&mut packet)? as usize;
        if length == 0 || length > packet.payload.len() || length > total_length - bytes_read {
            return Err(Error::TransferError);
        }

        writer.write_all(&packet.payload[..length])?;
        bytes_read += length;
        update_progress(bytes_read, total_length);
    }

    Ok(())
}

pub trait IspCommand: Interface {
    fn query_runtime_environment(&self, id: RuntimeEnvironment) -> Result<(), Error> {
        let mut packet: Packet = QueryRuntimeEnvironment::new(id).into();
        self.write(&packet, mem::size_of::<QueryRuntimeEnvironment>() as u16)?;
        self.read(&mut packet)?;
        todo!("data matching and error handling");
        Ok(())
    }
    /// Configure memory, using configuration block in RAM
    ///
    /// # Arguments
    ///
    /// * `memory_id`: Memory ID to be configure
    /// * `cfg_addr`: Configuration block address in RAM
    ///
    /// # Example
    ///
    /// ```ignore
    /// device.configure_memory(MemoryId::XPI0, 0x200);
    /// ```
    fn configure_memory(&self, memory_id: MemoryId, cfg_addr: u32) -> Result<(), Error> {
        let mut packet: Packet = ConfigureMemory::new(cfg_addr, memory_id).into();
        self.write(&packet, mem::size_of::<ConfigureMemory>() as u16)?;
        self.read(&mut packet)?;
        let resp = GenericCommandResponse::read_from_prefix(&packet.payload[..]).unwrap();
        resp.into()
    }

    fn write_memory<F>(
        &self,
        memory_id: MemoryId,
        offset: u32,
        data: &[u8],
        update_progress: F,
    ) -> Result<(), Error>
    where
        F: Fn(usize, usize),
    {
        write_from_reader(self, memory_id, offset, data.len(), data, update_progress)
    }

    fn read_memory<F>(
        &self,
        memory_id: MemoryId,
        offset: u32,
        data: &mut [u8],
        update_progress: F,
    ) -> Result<(), Error>
    where
        F: Fn(usize, usize),
    {
        read_to_writer(self, memory_id, offset, data.len(), data, update_progress)
    }

    fn write_file<P, F>(
        &self,
        path: P,
        memory_id: MemoryId,
        offset: u32,
        update_progress: F,
    ) -> Result<(), Error>
    where
        P: AsRef<Path>,
        F: Fn(usize, usize),
    {
        let file = File::open(path)?;
        let file_info = file.metadata()?;
        write_from_reader(
            self,
            memory_id,
            offset,
            file_info.len() as usize,
            file,
            update_progress,
        )
    }

    fn read_file<P, F>(
        &self,
        path: P,
        memory_id: MemoryId,
        offset: u32,
        total_length: usize,
        update_progress: F,
    ) -> Result<(), Error>
    where
        P: AsRef<Path>,
        F: Fn(usize, usize),
    {
        let file = File::create(path)?;
        read_to_writer(self, memory_id, offset, total_length, file, update_progress)
    }
}

#[derive(Debug)]
pub enum Error {
    Nak,
    TransferError,
    Timeout,
    IoError(io::Error),
    Other(u32),
}

impl Error {
    fn as_str(&self) -> &'static str {
        match self {
            Error::Nak => "negative acknowledge",
            Error::TransferError => "transfer error",
            Error::Timeout => "timeout",
            Error::IoError(_) => "io error",
            Error::Other(_) => "other error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Error::IoError(e) => write!(f, "{}: {}", self.as_str(), e),
            Error::Other(code) => write!(f, "{}: {}", self.as_str(), code),
            _ => write!(f, "{}", self.as_str()),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.as_str()
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoError(e)
    }
}
