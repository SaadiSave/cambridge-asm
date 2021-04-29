#![warn(clippy::pedantic)]

#[macro_use]
extern crate pest_derive;

use std::path::PathBuf;

use clap::{App, load_yaml};

fn main() {
    let yaml = load_yaml!("cli.yml");    
    let opts = App::from_yaml(yaml).get_matches();
    
    let input = opts.value_of_os("INPUT").unwrap();

    let path = PathBuf::from(input);

    let mut exec = parse::parse(&path);

    exec.exec();

    println!("Hello, world!");
}

mod exec;
mod parse;
