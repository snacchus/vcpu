use byteorder::{ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::mem;
use std::path::Path;
use util::Endian;

// TODO: use proper binary serialization using serde/bincode

#[derive(Debug, PartialEq)]
pub struct Executable {
    data_offset: u32,
    instructions: Vec<u8>,
    data: Vec<u8>,
}

impl Executable {
    pub fn from(data_offset: u32, instructions: Vec<u8>, data: Vec<u8>) -> Executable {
        Executable {
            data_offset,
            instructions,
            data,
        }
    }

    pub fn copy_from(data_offset: u32, instructions: &[u8], data: &[u8]) -> Executable {
        Executable {
            data_offset,
            instructions: Vec::from(instructions),
            data: Vec::from(data),
        }
    }

    pub fn data_offset(&self) -> u32 {
        self.data_offset
    }

    pub fn instructions(&self) -> &[u8] {
        &self.instructions[..]
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }

    pub fn required_size(&self) -> usize {
        mem::size_of::<u32>() * 3 + self.instructions.len() + self.data.len()
    }
}

pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Executable> {
    let instr_len = reader.read_u32::<Endian>()?;
    let data_length = reader.read_u32::<Endian>()?;
    let data_offset = reader.read_u32::<Endian>()?;

    let mut instructions = vec![0; instr_len as usize];
    let mut data = vec![0; data_length as usize];

    reader.read_exact(&mut instructions)?;
    reader.read_exact(&mut data)?;

    Ok(Executable::from(data_offset, instructions, data))
}

pub fn write<W: Write>(writer: &mut W, executable: &Executable) -> std::io::Result<()> {
    writer.write_u32::<Endian>(executable.instructions.len() as u32)?;
    writer.write_u32::<Endian>(executable.data.len() as u32)?;
    writer.write_u32::<Endian>(executable.data_offset)?;
    writer.write_all(&executable.instructions[..])?;
    writer.write_all(&executable.data[..])?;
    Ok(())
}

pub trait ReadVexExt: Read + Sized {
    fn read_vex(&mut self) -> std::io::Result<Executable> {
        read(self)
    }
}

impl<R: Read + Sized> ReadVexExt for R {}

pub trait WriteVexExt: Write + Sized {
    fn write_vex(&mut self, executable: &Executable) -> std::io::Result<()> {
        write(self, executable)
    }
}

impl<W: Write + Sized> WriteVexExt for W {}

pub fn read_file<P: AsRef<Path>>(path: P) -> std::io::Result<Executable> {
    BufReader::new(File::open(path)?).read_vex()
}

pub fn write_file<P: AsRef<Path>>(path: P, executable: &Executable) -> std::io::Result<()> {
    BufWriter::new(File::create(path)?).write_vex(executable)
}

#[cfg(test)]
mod test;
