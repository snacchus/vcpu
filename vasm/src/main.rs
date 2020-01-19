#[macro_use]
extern crate clap;

use clap::Arg;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum IOErrorContext {
    ReadInput,
    WriteOutput,
}

#[derive(Debug)]
enum Error {
    VASM(vasm::Error),
    IO(std::io::Error, IOErrorContext, PathBuf),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(err, context, path) => writeln!(
                f,
                "{} file \"{}\" failed: {}",
                match context {
                    IOErrorContext::ReadInput => "Reading input",
                    IOErrorContext::WriteOutput => "Writing output",
                },
                path.display(),
                err
            ),
            Error::VASM(err) => {
                writeln!(f, "Parsing input failed:")?;
                write!(f, "{}", err)
            }
        }
    }
}

fn main() {
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

    if let Err(err) = vasm(input, output) {
        eprintln!("{}", err);
    }
}

fn vasm(input: &str, output: Option<&str>) -> Result<(), Error> {
    let input_path = Path::new(input);

    // Read input file
    let input_file = File::open(input_path)
        .map_err(|err| Error::IO(err, IOErrorContext::ReadInput, input_path.to_owned()))?;
    let mut buf_reader = BufReader::new(input_file);
    let mut input = String::new();

    buf_reader
        .read_to_string(&mut input)
        .map_err(|err| Error::IO(err, IOErrorContext::ReadInput, input_path.to_owned()))?;

    // Perform parse
    let program = vasm::assemble(&input).map_err(|err| {
        Error::VASM(match input_path.to_str() {
            Some(path_str) => err.with_path(path_str),
            None => err,
        })
    })?;

    let output_path: PathBuf = output
        .map(PathBuf::from)
        .unwrap_or_else(|| input_path.with_extension("vex"));

    // Write output file
    vexfile::write_file(&output_path, &program)
        .map_err(|err| Error::IO(err, IOErrorContext::WriteOutput, output_path))?;
    Ok(())
}
