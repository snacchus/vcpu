use std::{error::Error, fmt};

pub type Endian = byteorder::LittleEndian;

pub trait InteropGetName {
    fn interop_name(&self) -> &'static [u8];
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ParseEnumError {
    pub value: String,
    pub enum_name: &'static str,
}

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Failed to parse \"{}\" as {}.",
            &self.value, &self.enum_name
        )
    }
}

impl Error for ParseEnumError {
    fn description(&self) -> &str {
        "Failed to parse enum."
    }
}

pub trait EnumFromStr: Sized {
    fn from_str(s: &str) -> Result<Self, ParseEnumError>;
}
