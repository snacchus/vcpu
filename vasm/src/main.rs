#[macro_use]
extern crate clap;

use byteorder::WriteBytesExt;
use clap::Arg;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use util::Endian;
use vasm::SourceMapItem;

#[derive(Debug)]
enum IOErrorContext {
    ReadInput,
    WriteOutput,
}

#[derive(Debug)]
enum Error {
    Vasm(vasm::Error),
    Io(std::io::Error, IOErrorContext, PathBuf),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err, context, path) => writeln!(
                f,
                "{} file \"{}\" failed: {}",
                match context {
                    IOErrorContext::ReadInput => "Reading input",
                    IOErrorContext::WriteOutput => "Writing output",
                },
                path.display(),
                err
            ),
            Error::Vasm(err) => {
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
        .arg(
            Arg::with_name("source_map")
                .short("m")
                .long("source_map")
                .takes_value(true)
                .value_name("SOURCE_MAP")
                .help("Sets the file to write the source map to"),
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let output = matches.value_of("OUTPUT");
    let map = matches.value_of("SOURCE_MAP");

    if let Err(err) = vasm(input, output, map) {
        eprintln!("{}", err);
    }
}

fn vasm(input: &str, output: Option<&str>, map: Option<&str>) -> Result<(), Error> {
    let input_path = Path::new(input);

    // Read input file
    let input_file = File::open(input_path)
        .map_err(|err| Error::Io(err, IOErrorContext::ReadInput, input_path.to_owned()))?;
    let mut buf_reader = BufReader::new(input_file);
    let mut input = String::new();

    buf_reader
        .read_to_string(&mut input)
        .map_err(|err| Error::Io(err, IOErrorContext::ReadInput, input_path.to_owned()))?;

    // Perform parse
    let (executable, source_map) = vasm::assemble(&input).map_err(|err| {
        Error::Vasm(match input_path.to_str() {
            Some(path_str) => err.with_path(path_str),
            None => err,
        })
    })?;

    let output_path: PathBuf = output
        .map(PathBuf::from)
        .unwrap_or_else(|| input_path.with_extension("vex"));

    // Write output file
    vex::write_file(&output_path, &executable)
        .map_err(|err| Error::Io(err, IOErrorContext::WriteOutput, output_path))?;

    // Write source map file (if path is set)
    if let Some(map_path_str) = map {
        let map_path = PathBuf::from(map_path_str);
        write_source_map(&source_map[..], &map_path)
            .map_err(|err| Error::Io(err, IOErrorContext::WriteOutput, map_path))?;
    }
    Ok(())
}

fn write_source_map(source_map: &[SourceMapItem], path: &PathBuf) -> std::io::Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    for item in source_map.iter() {
        writer.write_u32::<Endian>(item.start_line)?;
        writer.write_u32::<Endian>(item.line_count)?;
    }
    Ok(())
}
