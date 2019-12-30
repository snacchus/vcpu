use std::fs::File;
use std::io::prelude::Read;
use std::io::BufReader;
use std::path::Path;
use vcpu::{ExitCode, Processor};
use vexfile::Program;

#[derive(Debug)]
pub enum Error {
    VASM(vasm::Error),
    IO(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IO(err)
    }
}

pub fn run_program(program: &Program, mem_size: u32) -> Result<(Processor, ExitCode), Error> {
    let total_mem_size = program.data().len() as u32 + mem_size;
    let mut memory = vec![0; total_mem_size as usize];

    let mut processor = Processor::default();

    let exit_code = processor.run(program.instructions(), &mut memory);

    Ok((processor, exit_code))
}

pub fn run_vexfile<P: AsRef<Path>>(path: P, mem_size: u32) -> Result<(Processor, ExitCode), Error> {
    let program = vexfile::read_file(path)?;
    run_program(&program, mem_size)
}

pub fn run_vasm<P: AsRef<Path>>(path: P, mem_size: u32) -> Result<(Processor, ExitCode), Error> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut input = String::new();
    buf_reader.read_to_string(&mut input)?;

    let program = vasm::parse_and_assemble(&input).map_err(Error::VASM)?;

    run_program(&program, mem_size)
}
