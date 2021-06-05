// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![warn(clippy::pedantic)]

use cambridge_asm::parse;
use clap::{load_yaml, App};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::path::PathBuf;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let opts = App::from_yaml(yaml).get_matches();

    let input = opts.value_of_os("INPUT").unwrap();

    let verbosity = opts.occurrences_of("verbose");

    #[cfg(not(debug_assertions))]
    std::panic::set_hook(Box::new(handle_panic));

    SimpleLogger::new()
        .with_level(get_log_level(verbosity))
        .init()
        .unwrap();

    let mut x = opts.is_present("perf").then(std::time::Instant::now);

    let path = PathBuf::from(input);

    let mut exec = parse::parse(&path);

    x.is_some().then(|| {
        println!(
            "Total parse time (incl. executor creation): {:?}\nExecution starts here:",
            x.unwrap().elapsed()
        );
        x = Some(std::time::Instant::now())
    });

    exec.exec();

    x.is_some()
        .then(|| println!("Execution done.\nExecution time: {:?}", x.unwrap().elapsed()));
}

fn get_log_level(v: u64) -> LevelFilter {
    match v {
        0 => LevelFilter::Off,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}

#[cfg(not(debug_assertions))]
fn handle_panic(info: &std::panic::PanicInfo) {
    if let Some(l) = info.location() {
        println!("Panic occured at {}:{} - \"", l.file(), l.line())
    } else {
        println!("Panic occured, unable to determine location - \"")
    }

    if let Some(msg) = info.payload().downcast_ref::<String>() {
        println!("{}\n\"\nTo debug, try increasing the verbosity by passing -v flags if the error message is unclear.\nOpen an issue on github if the panic appears to be an internal error.", msg)
    }
}
