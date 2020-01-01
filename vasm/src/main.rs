#[macro_use]
extern crate clap;

use clap::Arg;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum Error {
    VASM(vasm::Error),
    IO(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IO(err)
    }
}

// TODO: impl fmt::Display for Error

fn main() -> Result<(), Error> {
    // Parse command line arguments
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .value_name("OUTPUT")
                .help("Sets the output file to write to"),
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let output = matches.value_of("OUTPUT");

    vasm(input, output)
}

fn vasm(input: &str, output: Option<&str>) -> Result<(), Error> {
    let input_path = Path::new(input);

    // Read input file
    let input_file = File::open(input_path)?;
    let mut buf_reader = BufReader::new(input_file);
    let mut input = String::new();
    buf_reader.read_to_string(&mut input)?;

    // Perform parse
    // TODO: Proper error reporting
    let program = vasm::assemble(&input).map_err(Error::VASM)?;

    let output_path: PathBuf = output
        .map(PathBuf::from)
        .unwrap_or_else(|| input_path.with_extension("vex"));

    // Write output file
    vexfile::write_file(output_path, &program)?;
    Ok(())
}
