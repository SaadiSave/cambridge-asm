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
pub mod inst;
pub mod parse;

#[cfg(feature = "compile")]
pub mod compile;

#[cfg(test)]
#[cfg(feature = "cambridge")]
const PROGRAMS: [(&str, usize, &[u8]); 1] =
    [(include_str!("../examples/hello.pasm"), 207, b"HELLO\n")];

#[cfg(test)]
#[cfg(not(feature = "cambridge"))]
const PROGRAMS: [(&str, usize, &[u8]); 4] = [
    (include_str!("../examples/division.pasm"), 65, b"5\nA\n"),
    (
        include_str!("../examples/multiplication.pasm"),
        15625,
        b"15625\n",
    ),
    (include_str!("../examples/hello.pasm"), 207, b"HELLO\n"),
    (include_str!("../examples/functions.pasm"), 65, b"A"),
];

#[cfg(test)]
use std::{io::Write, rc::Rc, sync::RwLock};

#[cfg(test)]
#[derive(Clone)]
#[repr(transparent)]
pub struct TestStdout(Rc<RwLock<Vec<u8>>>);

#[cfg(test)]
impl TestStdout {
    pub fn new(s: impl std::ops::Deref<Target = [u8]>) -> Self {
        Self(Rc::new(RwLock::new(s.to_vec())))
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.0.read().unwrap().clone()
    }
}

#[cfg(test)]
impl Write for TestStdout {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write().unwrap().extend(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
