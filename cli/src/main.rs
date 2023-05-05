// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![warn(clippy::pedantic)]

use cambridge_asm::{
    compile::{self, CompiledProg},
    exec::Io,
    parse::{self, DefaultSet},
};
use clap::{ArgEnum, Parser};
use std::ffi::OsString;

#[derive(Parser)]
#[clap(name = "Cambridge Pseudoassembly Interpreter")]
#[clap(version = concat!(env!("CARGO_PKG_VERSION"), "\n", "Library version 0.18.0"))]
#[clap(author = "Saadi Save <github.com/SaadiSave>")]
#[clap(about = "Run pseudoassembly from Cambridge International syllabus 9618 (2021)")]
enum Commands {
    /// Run compiled or plaintext pseudoassembly
    Run {
        /// Path to the input file containing compiled or plaintext pseudoassembly
        path: OsString,

        /// Increase logging level
        #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
        verbosity: usize,

        /// Show execution time
        #[clap(short = 't', long = "bench")]
        bench: bool,

        /// Format of input file
        #[clap(arg_enum)]
        #[clap(short = 'f', long = "format")]
        #[clap(default_value_t = InFormats::Pasm)]
        format: InFormats,
    },
    /// Compile pseudoassembly
    Compile {
        /// Path to the input file containing pseudoassembly
        input: OsString,

        /// Path to output file
        #[clap(short = 'o', long = "output")]
        output: Option<OsString>,

        /// Increase logging level
        #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
        verbosity: usize,

        /// Format of output file
        #[clap(arg_enum)]
        #[clap(short = 'f', long = "format")]
        #[clap(default_value_t = OutFormats::Json)]
        format: OutFormats,

        /// Minify output
        #[clap(short = 'm', long = "minify")]
        minify: bool,

        /// Include debuginfo
        #[clap(short, long)]
        debug: bool,
    },
}

#[derive(ArgEnum, Clone)]
enum InFormats {
    Pasm,
    Json,
    Ron,
    Yaml,
    Bin,
}

#[derive(ArgEnum, Clone)]
enum OutFormats {
    Json,
    Ron,
    Yaml,
    Bin,
}

fn main() -> anyhow::Result<()> {
    #[cfg(not(debug_assertions))]
    std::panic::set_hook(Box::new(handle_panic));

    let command = Commands::parse();

    let io = Io::default();

    match command {
        Commands::Run {
            path,
            verbosity,
            bench,
            format,
        } => run(path, verbosity, bench, format, io)?,
        Commands::Compile {
            input,
            output,
            verbosity,
            format,
            minify,
            debug,
        } => compile(input, output, verbosity, format, minify, debug)?,
    }

    Ok(())
}

#[allow(clippy::enum_glob_use, clippy::needless_pass_by_value)]
fn run(
    path: OsString,
    verbosity: usize,
    bench: bool,
    format: InFormats,
    io: Io,
) -> anyhow::Result<()> {
    use InFormats::*;

    init_logger(verbosity);

    let prog_bytes = std::fs::read(path)?;

    let mut timer = bench.then(std::time::Instant::now);

    let mut executor = match format {
        Pasm => parse::jit::<DefaultSet>(String::from_utf8_lossy(&prog_bytes), io).unwrap(),
        Json => serde_json::from_str::<CompiledProg>(&String::from_utf8_lossy(&prog_bytes))?
            .to_executor::<DefaultSet>(io),
        Ron => ron::from_str::<CompiledProg>(&String::from_utf8_lossy(&prog_bytes))?
            .to_executor::<DefaultSet>(io),
        Yaml => serde_yaml::from_str::<CompiledProg>(&String::from_utf8_lossy(&prog_bytes))?
            .to_executor::<DefaultSet>(io),
        Bin => {
            bincode::decode_from_slice::<CompiledProg, _>(&prog_bytes, bincode::config::standard())?
                .0
                .to_executor::<DefaultSet>(io)
        }
    };

    timer = timer.map(|t| {
        println!("Total parse time: {:?}", t.elapsed());
        std::time::Instant::now()
    });

    if timer.is_some() || verbosity > 0 {
        println!("Execution starts on next line");
    }

    executor.exec::<DefaultSet>();

    if let Some(t) = timer {
        println!("Execution done\nExecution time: {:?}", t.elapsed());
    }

    Ok(())
}

#[allow(clippy::enum_glob_use, clippy::needless_pass_by_value)]
fn compile(
    mut input: OsString,
    output: Option<OsString>,
    verbosity: usize,
    format: OutFormats,
    minify: bool,
    debug: bool,
) -> std::io::Result<()> {
    use OutFormats::*;

    init_logger(verbosity);

    let prog = std::fs::read_to_string(&input)?;

    let compiled = compile::compile::<DefaultSet>(prog, debug).unwrap();

    let output_path = output.unwrap_or_else(|| {
        input.push(match format {
            Json => ".json",
            Ron => ".ron",
            Yaml => ".yaml",
            Bin => ".bin",
        });
        input
    });

    let serialised = match format {
        Json => {
            use serde_json::ser::{to_string, to_string_pretty};

            {
                if minify {
                    to_string(&compiled).unwrap()
                } else {
                    to_string_pretty(&compiled).unwrap()
                }
            }
            .into_bytes()
        }
        Ron => {
            use ron::ser::{to_string, to_string_pretty, PrettyConfig};

            {
                if minify {
                    to_string(&compiled).unwrap()
                } else {
                    to_string_pretty(&compiled, PrettyConfig::default()).unwrap()
                }
            }
            .into_bytes()
        }
        Yaml => serde_yaml::to_string(&compiled).unwrap().into_bytes(),
        Bin => bincode::encode_to_vec(&compiled, bincode::config::standard()).unwrap(),
    };

    std::fs::write(output_path, serialised)
}

fn init_logger(verbosity: usize) {
    set_log_level(verbosity);
    env_logger::builder()
        .format_timestamp(None)
        .format_indent(None)
        .format_target(false)
        .init();
}

fn set_log_level(v: usize) {
    use std::env;
    match v {
        0 => env::set_var("RUST_LOG", "off"),
        1 => env::set_var("RUST_LOG", "warn"),
        2 => env::set_var("RUST_LOG", "info"),
        3 => env::set_var("RUST_LOG", "debug"),
        _ => env::set_var("RUST_LOG", "trace"),
    }
}

#[cfg(not(debug_assertions))]
fn handle_panic(info: &std::panic::PanicInfo) {
    if let Some(l) = info.location() {
        println!(
            "Program panicked (crashed). Panic occurred at {}:{}",
            l.file(),
            l.line()
        );
    } else {
        println!("Program panicked (crashed). Unable to locate the source of the panic.");
    }

    if let Some(msg) = info.payload().downcast_ref::<&str>() {
        println!("\n'{msg}'\n");
    } else if let Some(msg) = info.payload().downcast_ref::<String>() {
        println!("\n'{msg}'\n");
    }

    println!("To debug, try increasing the verbosity by passing -v flags if the error message is unclear.\nOpen an issue on github if the panic appears to be an internal error.");
}
