// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![warn(clippy::pedantic)]

use cambridge_asm::parse;
use clap::Parser;
use std::{ffi::OsString, path::PathBuf};

#[derive(Parser)]
#[clap(name = "Cambridge Pseudoassembly Interpreter")]
#[clap(version = "0.11.2")]
#[clap(author = "Saadi Save <github.com/SaadiSave>")]
#[clap(about = "Run pseudoassembly from Cambridge International syllabus 9618 (2021)")]
struct Cli {
    #[clap(help = "Path to the input file containing pseudoassembly")]
    input: OsString,

    #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
    #[clap(help = "Increase logging level")]
    verbosity: usize,

    #[clap(short = 't', long = "bench")]
    #[clap(help = "Show execution time")]
    bench: bool,
}

fn main() {
    let parsed = Cli::parse();

    set_log_level(parsed.verbosity);

    #[cfg(not(debug_assertions))]
    std::panic::set_hook(Box::new(handle_panic));

    env_logger::builder()
        .format_timestamp(None)
        .format_indent(None)
        .format_module_path(false)
        .init();

    let mut x = parsed.bench.then(std::time::Instant::now);

    let fpath = PathBuf::from(parsed.input);

    #[cfg(feature = "cambridge")]
    let mut exec = parse::from_file(&fpath, parse::get_fn);

    #[cfg(not(feature = "cambridge"))]
    let mut exec = parse::from_file(&fpath, parse::get_fn_ext);

    if let Some(inst) = x {
        println!("Total parse time: {:?}", inst.elapsed());
        x = Some(std::time::Instant::now());
    }

    if x.is_some() || parsed.verbosity > 0 {
        println!("Execution starts on next line");
    }

    exec.exec();

    if let Some(inst) = x {
        println!("Execution done\nExecution time: {:?}", inst.elapsed());
    }
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
            "Program panicked (crashed). Panic occurred at {}:{} -",
            l.file(),
            l.line()
        );
    } else {
        println!("Program panicked (crashed). Unable to locate the source of the panic -");
    }

    if let Some(msg) = info.payload().downcast_ref::<String>() {
        println!("{msg}\n\nTo debug, try increasing the verbosity by passing -v flags if the error message is unclear.\nOpen an issue on github if the panic appears to be an internal error.");
    }
}
