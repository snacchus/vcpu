#[macro_use]
extern crate clap;

use clap::{Arg, ArgGroup};

#[derive(Debug)]
enum Error {
    Vex(vex::Error),
    Clap(clap::Error),
}

fn main() -> Result<(), Error> {
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("PROGRAM")
                .help("Sets the program file to use")
                .index(1),
        )
        .arg(
            Arg::with_name("assembly")
                .short("a")
                .long("assembly")
                .takes_value(true)
                .value_name("ASSEMBLY")
                .help("Sets the assembly file to use"),
        )
        .arg(
            Arg::with_name("memory")
                .short("m")
                .long("memory")
                .takes_value(true)
                .value_name("MEMORY")
                .default_value("1024")
                .required(false)
                .help("Sets the size of the memory"),
        )
        .group(
            ArgGroup::with_name("input")
                .args(&["PROGRAM", "assembly"])
                .required(true),
        )
        .get_matches();

    let mem_size = value_t!(matches.value_of("memory"), u32).map_err(Error::Clap)?;

    let (_processor, exit_code) = match matches.value_of("PROGRAM") {
        Some(program) => vex::run_vexfile(program, mem_size),

        None => {
            let asm = matches.value_of("assembly").unwrap();
            vex::run_vasm(asm, mem_size)
        }
    }
    .map_err(Error::Vex)?;

    println!("Exit code: {:?}", exit_code);

    Ok(())
}
