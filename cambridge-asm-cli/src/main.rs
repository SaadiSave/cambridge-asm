// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![warn(clippy::pedantic)]

use cambridge_asm::{
    compile::{self, CompiledProg},
    exec::Executor,
    parse::{self, InstSet},
};
use clap::{ArgEnum, Parser, Subcommand};
use std::ffi::OsString;

#[cfg(feature = "cambridge")]
const INST_SET: InstSet = parse::get_fn;

#[cfg(not(feature = "cambridge"))]
const INST_SET: InstSet = parse::get_fn_ext;

#[derive(Parser)]
#[clap(name = "Cambridge Pseudoassembly Interpreter")]
#[clap(version = "0.12")]
#[clap(author = "Saadi Save <github.com/SaadiSave>")]
#[clap(about = "Run pseudoassembly from Cambridge International syllabus 9618 (2021)")]
struct Cli {
    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run compiled or plaintext pseudoassembly
    Run {
        #[clap(help = "Path to the input file containing compiled or plaintext pseudoassembly")]
        path: OsString,

        #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
        #[clap(help = "Increase logging level")]
        verbosity: usize,

        #[clap(short = 't', long = "bench")]
        #[clap(help = "Show execution time")]
        bench: bool,

        #[clap(arg_enum)]
        #[clap(default_value_t = InFormats::Pasm)]
        #[clap(short = 'f', long = "format")]
        #[clap(help = "Format of input file")]
        format: InFormats,
    },
    /// Compile pseudoassembly
    Compile {
        #[clap(help = "Path to the input file containing pseudoassembly")]
        input: OsString,

        #[clap(short = 'o', long = "output")]
        #[clap(help = "Path to output file")]
        output: Option<OsString>,

        #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
        #[clap(help = "Increase logging level")]
        verbosity: usize,

        #[clap(arg_enum)]
        #[clap(short = 'f', long = "format")]
        #[clap(help = "Format of output file")]
        #[clap(default_value_t = OutFormats::Json)]
        format: OutFormats,

        #[clap(short = 'm', long = "minify")]
        #[clap(help = "Minify output")]
        minify: bool,
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

#[allow(clippy::enum_glob_use)]
fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    #[cfg(not(debug_assertions))]
    std::panic::set_hook(Box::new(handle_panic));

    match cli.commands {
        Commands::Run {
            path,
            verbosity,
            bench,
            format,
        } => {
            use InFormats::*;

            let parser: Box<dyn FnOnce(Vec<u8>, InstSet) -> Executor> = match format {
                Pasm => Box::new(|v, set| parse::parse(String::from_utf8_lossy(&v), set)),
                Json => Box::new(|v, set| {
                    serde_json::from_str::<CompiledProg>(&String::from_utf8_lossy(&v))
                        .unwrap()
                        .to_executor(set)
                }),
                Ron => Box::new(|v, set| {
                    ron::from_str::<CompiledProg>(&String::from_utf8_lossy(&v))
                        .unwrap()
                        .to_executor(set)
                }),
                Yaml => Box::new(|v, set| {
                    serde_yaml::from_str::<CompiledProg>(&String::from_utf8_lossy(&v))
                        .unwrap()
                        .to_executor(set)
                }),
                Bin => Box::new(|v, set| {
                    bincode::decode_from_slice::<CompiledProg, _>(&v, bincode::config::standard())
                        .unwrap()
                        .0
                        .to_executor(set)
                }),
            };

            init_logger(verbosity);
            let prog_bytes = std::fs::read(path)?;
            let mut timer = bench.then(std::time::Instant::now);

            let mut executor = parser(prog_bytes, INST_SET);

            timer = timer.map(|t| {
                println!("Total parse time: {:?}", t.elapsed());
                std::time::Instant::now()
            });
            if timer.is_some() || verbosity > 0 {
                println!("Execution starts on next line");
            }

            executor.exec();

            let _ = timer.map(|t| println!("Execution done\nExecution time: {:?}", t.elapsed()));
        }
        Commands::Compile {
            mut input,
            output,
            verbosity,
            format,
            minify,
        } => {
            use OutFormats::*;

            let serializer: Box<dyn FnOnce(CompiledProg) -> Vec<u8>> = match format {
                Json => Box::new(|prog| {
                    use serde_json::ser::{to_string, to_string_pretty};

                    (if minify {
                        to_string(&prog).unwrap()
                    } else {
                        to_string_pretty(&prog).unwrap()
                    })
                    .into_bytes()
                }),
                Ron => Box::new(|prog| {
                    use ron::ser::{to_string, to_string_pretty, PrettyConfig};

                    (if minify {
                        to_string(&prog).unwrap()
                    } else {
                        to_string_pretty(&prog, PrettyConfig::default()).unwrap()
                    })
                    .into_bytes()
                }),
                Yaml => Box::new(|prog| serde_yaml::to_string(&prog).unwrap().into_bytes()),
                Bin => Box::new(|prog| {
                    bincode::encode_to_vec(&prog, bincode::config::standard()).unwrap()
                }),
            };

            init_logger(verbosity);
            let prog = std::fs::read_to_string(&input)?;
            let compiled = compile::compile(prog, INST_SET);

            let output = output.unwrap_or_else(|| {
                input.push(match format {
                    Json => ".json",
                    Ron => ".ron",
                    Yaml => ".yaml",
                    Bin => ".bin",
                });
                input
            });

            std::fs::write(output, &*serializer(compiled))?;
        }
    }

    Ok(())
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
