use std::fs;
use std::io;
use std::path::Path;

use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use hpm_rt::{PinGroup, PortConnection, QuadIOEnableSequence, XpiNorConfigurationOption};

trait SelectPromptModel: Sized {
    const COUNT: u32;

    fn to_printable_str(&self) -> &'static str;
    fn from_num(num: u32) -> Option<Self>;

    fn show_select_prompt(prompt: &str) -> std::io::Result<Self> {
        let items: Vec<&str> = (0..Self::COUNT)
            .map(|num| Self::from_num(num).unwrap().to_printable_str())
            .collect();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .with_prompt(prompt)
            .default(0)
            .interact()?;
        Ok(Self::from_num(selection as u32).unwrap())
    }
}

impl SelectPromptModel for PinGroup {
    const COUNT: u32 = 2;

    fn from_num(num: u32) -> Option<Self> {
        match num {
            0 => Some(Self::Group1),
            1 => Some(Self::Group2),
            _ => None,
        }
    }

    fn to_printable_str(&self) -> &'static str {
        match self {
            PinGroup::Group1 => "Group 1",
            PinGroup::Group2 => "Group 2",
        }
    }
}

impl SelectPromptModel for PortConnection {
    const COUNT: u32 = 5;

    fn to_printable_str(&self) -> &'static str {
        match self {
            PortConnection::PortACs0 => "Port A with CS 0",
            PortConnection::PortBCs0 => "Port B with CS 0",
            PortConnection::PortACs0PortBCs0 => "Port A with CS 0, Port B with CS 0",
            PortConnection::PortACs0PortACs1 => "Port A with CS 0, Port A with CS 1",
            PortConnection::PortBCs0PortBCs1 => "Port B with CS 0, Port B with CS 1",
        }
    }

    fn from_num(num: u32) -> Option<Self> {
        match num {
            0 => Some(Self::PortACs0),
            1 => Some(Self::PortBCs0),
            2 => Some(Self::PortACs0PortBCs0),
            3 => Some(Self::PortACs0PortACs1),
            4 => Some(Self::PortBCs0PortBCs1),
            _ => None,
        }
    }
}

impl SelectPromptModel for QuadIOEnableSequence {
    const COUNT: u32 = 5;

    fn to_printable_str(&self) -> &'static str {
        match self {
            QuadIOEnableSequence::None => "None",
            QuadIOEnableSequence::Status1Bit6 => "Status Register 1 bit 6",
            QuadIOEnableSequence::Status2Bit1 => "Status Register 2 bit 1",
            QuadIOEnableSequence::Status2Bit7 => "Status Register 2 bit 7",
            QuadIOEnableSequence::Status2Bit1ProgrammedBy0x31 => {
                "Status Register 2 bit 1, programmed by 0x31"
            }
        }
    }

    fn from_num(num: u32) -> Option<Self> {
        match num {
            0 => Some(Self::None),
            1 => Some(Self::Status1Bit6),
            2 => Some(Self::Status2Bit1),
            3 => Some(Self::Status2Bit7),
            4 => Some(Self::Status2Bit1ProgrammedBy0x31),
            _ => None,
        }
    }
}

pub fn config_wizard<P>(path: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let config = XpiNorConfigurationOption::new();

    // 1. Select port connection
    let port = PortConnection::show_select_prompt("Select port connection")?;
    // 2. Select pin group
    let group = PinGroup::show_select_prompt("Select pin group")?;
    // 3. Select Quad Enable sequence
    let sequence = QuadIOEnableSequence::show_select_prompt("Select Quad Enable sequence")?;

    // Check if file exist
    if let Ok(_) = fs::metadata(&path) {
        let replace = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "{} already exists, overwrite it?",
                path.as_ref().to_str().unwrap()
            ))
            .interact()?;
        if !replace {
            return Ok(());
        }
    }

    let mut output_file = fs::File::create(&path)?;
    config
        .connect_port(port)
        .pin_group(group)
        .quad_io_enable_sequence(sequence)
        .write(&mut output_file)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

    println!(
        "Config file was successfully saved to: {}",
        fs::canonicalize(path)?.to_str().unwrap()
    );

    Ok(())
}
