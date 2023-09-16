// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![deny(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::items_after_test_module
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
pub(crate) mod test_stdio {
    include!("../test_stdio.rs");
}

#[cfg(test)]
pub(crate) use test_stdio::TestStdio;

#[cfg(test)]
#[cfg(not(feature = "extended"))]
const PROGRAMS: [(&str, usize, &[u8], &[u8]); 1] =
    [(include_str!("../examples/hello.pasm"), 207, b"", b"HELLO\n")];

#[cfg(test)]
#[cfg(feature = "extended")]
const PROGRAMS: [(&str, usize, &[u8], &[u8]); 5] = [
    (
        include_str!("../examples/division.pasm"),
        65,
        b"",
        b"5\nA\n",
    ),
    (
        include_str!("../examples/multiplication.pasm"),
        15625,
        b"",
        b"15625\n",
    ),
    (include_str!("../examples/hello.pasm"), 207, b"", b"HELLO\n"),
    (include_str!("../examples/functions.pasm"), 65, b"", b"A"),
    (
        include_str!("../examples/showoff.pasm"),
        68,
        b"DIANA",
        b"HELLO\n",
    ),
];
