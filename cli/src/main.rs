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
use clap::{Parser, ValueEnum};
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Parser)]
#[clap(name = "Cambridge Pseudoassembly Interpreter")]
#[clap(version = concat!("v", env!("CARGO_PKG_VERSION"), "\nCambridge Pseudoassembly v", include_str!(concat!(env!("OUT_DIR"), "/LIBRARY_VERSION"))))]
#[clap(author = "Saadi Save <github.com/SaadiSave>")]
#[clap(about = "Run pseudoassembly from Cambridge International syllabus 9618 (2021)")]
enum Commands {
    /// Run compiled or plaintext pseudoassembly
    Run {
        /// Path to the input file containing compiled or plaintext pseudoassembly
        path: PathBuf,

        /// Increase logging level
        #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
        verbosity: u8,

        /// Show execution time
        #[arg(short = 't', long = "bench")]
        bench: bool,

        /// Format of input file
        #[arg(value_enum)]
        #[arg(short = 'f', long = "format")]
        #[arg(default_value_t = InFormats::Pasm)]
        format: InFormats,
    },
    /// Compile pseudoassembly
    Compile {
        /// Path to the input file containing pseudoassembly
        input: PathBuf,

        /// Path to output file
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// Increase logging level
        #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
        verbosity: u8,

        /// Format of output file
        #[arg(value_enum)]
        #[arg(short = 'f', long = "format")]
        #[arg(default_value_t = OutFormats::Json)]
        format: OutFormats,

        /// Minify output
        #[arg(short = 'm', long = "minify")]
        minify: bool,

        /// Include debuginfo
        #[arg(short, long)]
        debug: bool,
    },
}

#[derive(ValueEnum, Clone)]
enum InFormats {
    Pasm,
    Json,
    Ron,
    Yaml,
    Cbor,
}

#[derive(ValueEnum, Clone)]
enum OutFormats {
    Json,
    Ron,
    Yaml,
    Cbor,
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
fn run(path: PathBuf, verbosity: u8, bench: bool, format: InFormats, io: Io) -> anyhow::Result<()> {
    use InFormats::*;

    init_logger(verbosity);

    let file = File::open(path)?;

    let mut timer = bench.then(std::time::Instant::now);

    let read_to_string = |mut f: File| -> std::io::Result<_> {
        #[allow(clippy::cast_possible_truncation)]
        let mut buf = String::with_capacity(f.metadata()?.len() as usize);
        f.read_to_string(&mut buf)?;
        Ok(buf)
    };

    let mut executor = match format {
        Pasm => parse::jit::<DefaultSet>(read_to_string(file)?, io).unwrap(),
        Json => serde_json::from_str::<CompiledProg>(&read_to_string(file)?)?
            .to_executor::<DefaultSet>(io),
        Ron => ron::from_str::<CompiledProg>(&read_to_string(file)?)?.to_executor::<DefaultSet>(io),
        Yaml => serde_yaml::from_str::<CompiledProg>(&read_to_string(file)?)?
            .to_executor::<DefaultSet>(io),
        Cbor => ciborium::from_reader::<CompiledProg, _>(file)?.to_executor::<DefaultSet>(io),
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
    mut input: PathBuf,
    output: Option<PathBuf>,
    verbosity: u8,
    format: OutFormats,
    minify: bool,
    debug: bool,
) -> anyhow::Result<()> {
    use OutFormats::*;

    init_logger(verbosity);

    let prog = std::fs::read_to_string(&input)?;

    let compiled = compile::compile::<DefaultSet>(prog, debug).unwrap();

    let output_path = output.unwrap_or_else(|| {
        let ext = match format {
            Json => "json",
            Ron => "ron",
            Yaml => "yaml",
            Cbor => "cbor",
        };
        input.set_extension(ext);
        input
    });

    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(output_path)?;

    let json = |w: File, v: &CompiledProg| {
        if minify {
            serde_json::to_writer(w, v)
        } else {
            serde_json::to_writer_pretty(w, v)
        }
    };

    let ron = |w: File, v: &CompiledProg| {
        if minify {
            ron::ser::to_writer(w, v)
        } else {
            ron::ser::to_writer_pretty(w, v, ron::ser::PrettyConfig::default())
        }
    };

    let yaml = |w: File, v: &CompiledProg| serde_yaml::to_writer(w, v);

    let cbor = |w: File, v: &CompiledProg| ciborium::ser::into_writer(v, w);

    match format {
        Json => json(file, &compiled)?,
        Ron => ron(file, &compiled)?,
        Yaml => yaml(file, &compiled)?,
        Cbor => cbor(file, &compiled)?,
    }

    Ok(())
}

fn init_logger(verbosity: u8) {
    set_log_level(verbosity);
    env_logger::builder()
        .format_timestamp(None)
        .format_indent(None)
        .format_target(false)
        .init();
}

fn set_log_level(v: u8) {
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
