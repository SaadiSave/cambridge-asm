// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![warn(clippy::pedantic)]

use cambridge_asm::parse;
use clap::{load_yaml, App};
use std::path::PathBuf;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let opts = App::from_yaml(yaml).get_matches();

    let input = opts.value_of_os("INPUT").unwrap();

    let path = PathBuf::from(input);

    let mut exec = parse::parse(&path);

    exec.exec();
}
