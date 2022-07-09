// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(clippy::module_name_repetitions)]

use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::Deref,
};

/// Represents all possible runtime errors
#[derive(Debug)]
pub enum PasmError {
    Str(String),
    InvalidUtf8Byte(usize),
    InvalidOperand,
    NoOpInst,
    NoOperand,
    InvalidMemoryLoc(usize),
    InvalidIndirectAddress(usize),
    InvalidIndexedAddress(usize, usize),
    InvalidMultiOp,
}

impl Display for PasmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use PasmError::*;

        match self {
            Str(s) => f.write_str(s),
            InvalidUtf8Byte(b) => f.write_fmt(format_args!("#x{b:X} is not a valid UTF-8 byte.")),
            InvalidOperand => f.write_str("Operand is not a memory address, register, or literal. If you wanted to use a label, please double-check the label."),
            NoOperand => f.write_str("Operand missing."),
            NoOpInst => f.write_str("Instruction takes no operand."),
            InvalidMemoryLoc(addr) => f.write_fmt(format_args!("Memory address {addr} does not exist.")),
            InvalidIndirectAddress(addr) => f.write_fmt(format_args!("The value at memory address {addr} does not point to a valid memory address")),
            InvalidIndexedAddress(addr, offset) => f.write_fmt(format_args!("The memory address {addr} offset by IX value {offset} is not a valid memory address ({addr} + {offset} = {})", addr + offset)),
            InvalidMultiOp => f.write_str("Operand sequence is invalid"),
        }
    }
}

impl Error for PasmError {}

impl<T: Deref<Target = str>> From<T> for PasmError {
    fn from(s: T) -> Self {
        PasmError::Str(s.to_string())
    }
}

/// Convenience type to work with [`PasmError`]
///
/// Comparable to [`std::io::Result`]
pub type PasmResult<T = ()> = Result<T, PasmError>;

/// Stores original source code during execution
#[derive(Debug)]
#[repr(transparent)]
pub struct Source(Vec<String>);

impl Source {
    pub fn handle_err(
        &self,
        write: &mut impl std::io::Write,
        err: &PasmError,
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
            f.write_fmt(format_args!("    {inst}\n"))?;
        }

        f.write_str("")
    }
}
