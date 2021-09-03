#![allow(clippy::module_name_repetitions)]

use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::Deref,
};

#[derive(Debug)]
pub enum PasmError {
    Str(String),
    InvalidUtf8Byte(u8),
    InvalidLiteral,
    InvalidOperand,
    NoOperand,
    InvalidMemoryLoc(String),
    InvalidIndirectAddress(usize),
    InvalidMultiOp,
}

impl Display for PasmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use PasmError::*;

        match self {
            Str(s) => f.write_str(s),
            InvalidUtf8Byte(b) => f.write_fmt(format_args!("The value in the ACC, `{}`, is not a valid UTF-8 byte.", b)),
            InvalidLiteral => f.write_str("Operand is not a decimal, hexadecimal, or binary number."),
            InvalidOperand => f.write_str("Operand is not a memory location, register, or literal. If you wanted to use a label, please double-check the label."),
            NoOperand => f.write_str("Operand missing."),
            InvalidMemoryLoc(l) => f.write_fmt(format_args!("Memory location `{}` does not exist.", l)),
            InvalidIndirectAddress(v) => f.write_fmt(format_args!("The value at the memory location, '{}', is not a valid memory location. If you wanted to use a label, please double-check the label.", v)),
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

#[repr(transparent)]
pub struct Source(Vec<String>);

impl Source {
    pub fn handle_err(&self, err: &PasmError, pos: usize) {
        let mut out = String::new();
        out.push_str("Runtime Error:\n");

        for (i, s) in self.0.iter().enumerate() {
            if pos == i {
                if let Some(prev) = self.0.get(i - 1) {
                    out.push_str(&format!(
                        "\n{num:>w$}    {}",
                        prev,
                        num = i,
                        w = self.whitespace()
                    ));
                }
                out.push_str(&format!(
                    "\n{num:>w$}    {} <-",
                    s,
                    num = i + 1,
                    w = self.whitespace()
                ));
                if let Some(next) = self.0.get(i + 1) {
                    out.push_str(&format!(
                        "\n{num:>w$}    {}",
                        next,
                        num = i + 2,
                        w = self.whitespace()
                    ));
                }
                out.push_str(&format!("\n\nmessage: {}", err));
                break;
            }
        }
        println!("{}", out);
    }

    fn whitespace(&self) -> usize {
        self.0.len().to_string().len()
    }
}

impl<T: Deref<Target = str>> From<T> for Source {
    fn from(s: T) -> Self {
        Source(s.to_string().lines().map(String::from).collect())
    }
}

impl Debug for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Program {\n")?;

        for inst in &self.0 {
            f.write_fmt(format_args!("\t{}\n", inst))?;
        }

        f.write_str("}\n")
    }
}
