// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![deny(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

#[macro_use]
extern crate log;

pub mod exec;
pub mod inst;
pub mod parse;

#[cfg(feature = "compile")]
pub mod compile;

#[cfg(test)]
#[cfg(not(feature = "extended"))]
const PROGRAMS: [(&str, usize, &[u8]); 1] =
    [(include_str!("../examples/hello.pasm"), 207, b"HELLO\n")];

#[cfg(test)]
#[cfg(feature = "extended")]
const PROGRAMS: [(&str, usize, &[u8]); 5] = [
    (include_str!("../examples/division.pasm"), 65, b"5\nA\n"),
    (
        include_str!("../examples/multiplication.pasm"),
        15625,
        b"15625\n",
    ),
    (include_str!("../examples/hello.pasm"), 207, b"HELLO\n"),
    (include_str!("../examples/functions.pasm"), 65, b"A"),
    (include_str!("../examples/showoff.pasm"), 0, b"HELLO\n"),
];

#[cfg(test)]
pub(crate) mod test_stdout {
    include!("../test_stdout.rs");
}

#[cfg(test)]
pub(crate) use test_stdout::TestStdout;
