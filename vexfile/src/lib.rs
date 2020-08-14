use byteorder::{ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::mem;
use std::path::Path;
use util::Endian;

// TODO: use proper binary serialization using serde/bincode

#[derive(Debug, PartialEq)]
pub struct Program {
    data_offset: u32,
    instructions: Vec<u8>,
    data: Vec<u8>,
}

impl Program {
    pub fn from(data_offset: u32, instructions: Vec<u8>, data: Vec<u8>) -> Program {
        Program {
            data_offset,
            instructions,
            data,
        }
    }

    pub fn copy_from(data_offset: u32, instructions: &[u8], data: &[u8]) -> Program {
        Program {
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
}

pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Program> {
    let instr_len = reader.read_u32::<Endian>()?;
    let data_length = reader.read_u32::<Endian>()?;
    let data_offset = reader.read_u32::<Endian>()?;

    let mut instructions = vec![0; instr_len as usize];
    let mut data = vec![0; data_length as usize];

    reader.read_exact(&mut instructions)?;
    reader.read_exact(&mut data)?;

    Ok(Program::from(data_offset, instructions, data))
}

pub fn write<W: Write>(writer: &mut W, program: &Program) -> std::io::Result<()> {
    writer.write_u32::<Endian>(program.instructions.len() as u32)?;
    writer.write_u32::<Endian>(program.data.len() as u32)?;
    writer.write_u32::<Endian>(program.data_offset)?;
    writer.write_all(&program.instructions[..])?;
    writer.write_all(&program.data[..])?;
    Ok(())
}

pub fn get_required_size(program: &Program) -> usize {
    mem::size_of::<u32>() * 3 + program.instructions.len() + program.data().len()
}

pub trait ReadVexExt: Read + Sized {
    fn read_vex(&mut self) -> std::io::Result<Program> {
        read(self)
    }
}

impl<R: Read + Sized> ReadVexExt for R {}

pub trait WriteVexExt: Write + Sized {
    fn write_vex(&mut self, program: &Program) -> std::io::Result<()> {
        write(self, program)
    }
}

impl<W: Write + Sized> WriteVexExt for W {}

pub fn read_file<P: AsRef<Path>>(path: P) -> std::io::Result<Program> {
    BufReader::new(File::open(path)?).read_vex()
}

pub fn write_file<P: AsRef<Path>>(path: P, program: &Program) -> std::io::Result<()> {
    BufWriter::new(File::create(path)?).write_vex(program)
}

#[cfg(test)]
mod test;
