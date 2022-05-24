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

#[derive(Debug)]
pub enum PasmError {
    Str(String),
    InvalidUtf8Byte(usize),
    InvalidLiteral,
    InvalidOperand,
    NoOpInst,
    NoOperand,
    InvalidMemoryLoc(String),
    InvalidIndirectAddress(usize),
    InvalidIndexedAddress(usize),
    InvalidMultiOp,
}

impl Display for PasmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use PasmError::*;

        match self {
            Str(s) => f.write_str(s),
            InvalidUtf8Byte(b) => f.write_fmt(format_args!("#x{b:X} is not a valid UTF-8 byte.")),
            InvalidLiteral => f.write_str("Operand is not a decimal, hexadecimal, or binary number."),
            InvalidOperand => f.write_str("Operand is not a memory location, register, or literal. If you wanted to use a label, please double-check the label."),
            NoOperand => f.write_str("Operand missing."),
            NoOpInst => f.write_str("Instruction takes no operand."),
            InvalidMemoryLoc(l) => f.write_fmt(format_args!("Memory location `{l}` does not exist.")),
            InvalidIndirectAddress(v) | InvalidIndexedAddress(v) => f.write_fmt(format_args!("The value at the memory location, '{v}', is not a valid memory location. If you wanted to use a label, please double-check the label.")),
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

pub type PasmResult = Result<(), PasmError>;

#[derive(Debug)]
#[repr(transparent)]
pub struct Source(Vec<String>);

impl Source {
    pub fn handle_err(&self, write: &mut impl std::io::Write, err: &PasmError, pos: usize) {
        let mk_line =
            |inst: &str, num: usize| format!("\n{num:>w$}    {inst}", w = self.whitespace());

        let mut out = String::new();
        out.push_str("Runtime Error:\n");

        for (i, s) in self.0.iter().enumerate() {
            if pos == i {
                if let Some(prev) = self.0.get(i - 1) {
                    out.push_str(&mk_line(prev, i));
                }
                out.push_str(&format!(
                    "\n{num:>w$}    {s} <-",
                    num = i + 1,
                    w = self.whitespace()
                ));
                if let Some(next) = self.0.get(i + 1) {
                    out.push_str(&format!(
                        "\n{num:>w$}    {next}",
                        num = i + 2,
                        w = self.whitespace()
                    ));
                }
                out.push_str(&format!("\n\nmessage: {err}"));
                break;
            }
        }
        writeln!(write, "{}", out).unwrap();
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
