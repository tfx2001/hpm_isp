use std::fmt::Display;

use hidapi::{HidApi, HidDevice, HidError};
use num_enum::FromPrimitive;
use strum::{EnumIter, IntoEnumIterator};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

use crate::isp_command::{Error, Interface, IspCommand, Packet};

#[derive(AsBytes, FromZeroes, FromBytes)]
#[repr(C, packed)]
struct HidPayloadPacket {
    dir: u8,
    packet_type: u8,
    length: u16,
    payload: [u8; 512],
}

#[derive(AsBytes, FromZeroes, FromBytes)]
#[repr(C, packed)]
struct HidAcknowledgement {
    dir: u8,
    packet_type: u8,
    length: u16,
}

#[repr(u8)]
pub enum Direction {
    HostToDevice = 0x01,
    DeviceToHost = 0x02,
}

#[derive(FromPrimitive)]
#[repr(u8)]
enum PacketType {
    Ack = 0xA1,
    Nak = 0xA2,
    Abort = 0xA3,
    Payload = 0xA5,
    #[default]
    Others,
}

impl HidPayloadPacket {
    fn new(length: u16, payload: [u8; 512]) -> Self {
        HidPayloadPacket {
            dir: Direction::HostToDevice as u8,
            packet_type: PacketType::Payload as u8,
            length,
            payload,
        }
    }
}

impl HidAcknowledgement {
    fn new(packet_type: PacketType) -> Self {
        HidAcknowledgement {
            dir: Direction::HostToDevice as u8,
            packet_type: packet_type as u8,
            length: 0,
        }
    }
}

#[derive(EnumIter, Clone, Copy)]
#[repr(u16)]
pub enum Family {
    HPM6700_6400 = 0x0001,
    HPM6300 = 0x0002,
    HPM6200 = 0x0003,
    HPM6800 = 0x0004,
    HPM5300 = 0x0005,
    HPM6E00 = 0x0006,
}

impl Family {
    pub fn pid() -> u16 {
        0x34b7
    }

    pub fn vid(&self) -> u16 {
        *self as u16
    }
}

impl Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Family::HPM6700_6400 => "HPM6700/6400",
            Family::HPM6300 => "HPM6300",
            Family::HPM6200 => "HPM6200",
            Family::HPM6800 => "HPM6800",
            Family::HPM5300 => "HPM5300",
            Family::HPM6E00 => "HPM6E00",
        };
        write!(f, "{name}")
    }
}

pub struct HpmDevice {
    device: HidDevice,
    family: Family,
}

impl HpmDevice {
    pub fn open() -> Result<Self, Box<dyn std::error::Error>> {
        let api = HidApi::new().unwrap();
        // Connect to device using its VID and PID
        let (device, family) = Family::iter()
            .find_map(|chip| match api.open(Family::pid(), chip.vid()).ok() {
                Some(device) => Some((device, chip)),
                None => None,
            })
            .ok_or("Can't find any HPMicro device")?;
        Ok(Self { device, family })
    }

    pub fn family(&self) -> Family {
        self.family
    }
}

impl Interface for HpmDevice {
    fn write(&self, packet: &Packet, length: u16) -> Result<(), Error> {
        let mut buffer: [u8; 512] = [0u8; 512];

        // Host command/data stage
        packet.write_to_prefix(&mut buffer[..]);
        let hid_packet = HidPayloadPacket::new(length + 4, buffer);
        self.device.write(hid_packet.as_bytes())?;

        // Device ACK/NAK/Abort stage
        let mut buffer: [u8; 516] = [0u8; 516];
        self.device.read(&mut buffer)?;
        let ack_packet: HidAcknowledgement =
            HidAcknowledgement::read_from_prefix(&buffer[..]).unwrap();

        match ack_packet.packet_type.into() {
            PacketType::Ack | PacketType::Abort => Ok(()),
            _ => Err(Error::Nak),
        }
    }

    fn read(&self, packet: &mut Packet) -> Result<u16, Error> {
        let mut buffer = [0u8; 516];

        // Device response stage
        self.device.read(&mut buffer)?;
        let response_packet: HidPayloadPacket =
            HidPayloadPacket::read_from_prefix(&buffer[..]).unwrap();

        // Host ACK/NAK/Abort stage
        let ack_packet = HidAcknowledgement::new(PacketType::Ack);
        self.device.write(ack_packet.as_bytes())?;

        *packet = Packet::read_from_prefix(&response_packet.payload[..]).unwrap();
        Ok(response_packet.length - 4)
    }
}

impl IspCommand for HpmDevice {}

impl From<HidError> for Error {
    fn from(_: HidError) -> Self {
        Error::TransferError
    }
}
