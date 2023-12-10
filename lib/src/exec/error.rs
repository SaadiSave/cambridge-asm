// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(clippy::module_name_repetitions)]

use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::Deref,
};
use thiserror::Error;

/// Represents all possible runtime errors
#[derive(Debug, Error)]
pub enum RtError {
    #[error("{0}")]
    Other(String),
    #[error("Unexpected I/O error, caused by: {0}")]
    IoError(#[from] std::io::Error),
    #[error("#x{0:X} is not a valid UTF-8 byte.")]
    InvalidUtf8Byte(usize),
    #[error("Operand is not a memory address, register, or literal")]
    InvalidOperand,
    #[error("No operand needed")]
    NoOpInst,
    #[error("Operand missing")]
    NoOperand,
    #[error("Invalid memory address `{0}`")]
    InvalidAddr(usize),
    #[error("Invalid indirect access address {redirect} at memory address {src}")]
    InvalidIndirectAddr { src: usize, redirect: usize },
    #[error("Invalid indexed access address `{}` from {src} + {offset}", .src +.offset)]
    InvalidIndexedAddr { src: usize, offset: usize },
    #[error("Invalid operand sequence")]
    InvalidMultiOp,
}

impl From<&'static str> for RtError {
    fn from(value: &'static str) -> Self {
        Self::Other(value.to_string())
    }
}

impl From<String> for RtError {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

pub type RtResult<T = ()> = Result<T, RtError>;

/// Stores original source code during execution
#[derive(Debug, Default, Clone)]
#[repr(transparent)]
pub struct Source(Vec<String>);

impl Source {
    pub fn handle_err(
        &self,
        write: &mut impl std::io::Write,
        err: &RtError,
        pos: usize,
    ) -> std::io::Result<()> {
        writeln!(write, "Runtime Error:")?;
        writeln!(write)?;

        for (i, s) in self.0.iter().enumerate() {
            if pos == i {
                if let Some(prev) = self.0.get(i - 1) {
                    writeln!(write, "{num:>w$}    {prev}", num = i, w = self.whitespace())?;
                }

                writeln!(
                    write,
                    "{num:>w$}    {s} <-",
                    num = i + 1,
                    w = self.whitespace()
                )?;

                if let Some(next) = self.0.get(i + 1) {
                    writeln!(
                        write,
                        "{num:>w$}    {next}",
                        num = i + 2,
                        w = self.whitespace()
                    )?;
                }

                writeln!(write)?;
                writeln!(write, "message: {err}")?;
                break;
            }
        }
        writeln!(write)
    }

    fn whitespace(&self) -> usize {
        self.0.len().to_string().len()
    }
}

impl<T: Deref<Target = str>> From<T> for Source {
    fn from(s: T) -> Self {
        Source(
            s.to_string()
                .lines()
                .filter(|&el| !el.starts_with("//"))
                .map(String::from)
                .collect(),
        )
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for inst in &self.0 {
            writeln!(f, "    {inst}")?;
        }

        Ok(())
    }
}
