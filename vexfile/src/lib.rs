use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::fs::File;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Program {
    data: Vec<u8>,
    instructions: Vec<u8>,
}

impl Program {
    pub fn from(data: Vec<u8>, instructions: Vec<u8>) -> Program {
        Program {
            data: data,
            instructions: instructions
        }
    }

    pub fn copy_from(data: &[u8], instructions: &[u8]) -> Program {
        Program {
            data: Vec::from(data),
            instructions: Vec::from(instructions)
        }
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }

    pub fn instructions(&self) -> &[u8] {
        &self.instructions[..]
    }
}

pub fn read<R: Read>(reader: &mut R) -> std::io::Result<Program> {
    // TODO: Define endianness in "common" crate
    let data_length = reader.read_u32::<LittleEndian>()?;
    let mut data = vec![0; data_length as usize];
    reader.read_exact(&mut data)?;

    let mut instructions = Vec::new();
    reader.read_to_end(&mut instructions)?;

    Ok(Program::from(data, instructions))
}

pub fn write<W: Write>(writer: &mut W, program: &Program) -> std::io::Result<()> {
    // TODO: Define endianness in "common" crate
    writer.write_u32::<LittleEndian>(program.data.len() as u32)?;
    writer.write_all(&program.data[..])?;
    writer.write_all(&program.instructions[..])?;
    Ok(())
}

pub trait ReadVexExt: Read + Sized {
    fn read_vex(&mut self) -> std::io::Result<Program> {
        read(self)
    }
}

impl<R: Read + Sized> ReadVexExt for R { }

pub trait WriteVexExt: Write + Sized {
    fn write_vex(&mut self, program: &Program) -> std::io::Result<()> {
        write(self, program)
    }
}

impl<W: Write + Sized> WriteVexExt for W { }

pub fn read_file<P: AsRef<Path>>(path: P) -> std::io::Result<Program> {
    BufReader::new(File::open(path)?).read_vex()
}

pub fn write_file<P: AsRef<Path>>(path: P, program: &Program) -> std::io::Result<()> {
    BufWriter::new(File::create(path)?).write_vex(program)
}
