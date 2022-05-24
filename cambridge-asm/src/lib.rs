// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate
)]

#[macro_use]
extern crate log;

pub mod exec;
pub mod parse;
pub mod inst;

#[cfg(feature = "compile")]
pub mod compile;

#[cfg(test)]
#[cfg(feature = "cambridge")]
const PROGRAMS: [(&str, usize); 1] = [
    (include_str!("../examples/hello.pasm"), 207),
];

#[cfg(test)]
#[cfg(not(feature = "cambridge"))]
const PROGRAMS: [(&str, usize); 4] = [
    (include_str!("../examples/division.pasm"), 65),
    (include_str!("../examples/multiplication.pasm"), 15625),
    (include_str!("../examples/hello.pasm"), 207),
    (include_str!("../examples/functions.pasm"), 65),
];
