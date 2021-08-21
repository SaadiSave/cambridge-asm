// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
    ops::Deref,
};

/// # Arithmetic
/// Module for arithmetic operations
pub mod arith;

/// # I/O
/// Module for input, output and debugging
pub mod io;

/// # Data movement
/// Module for moving data between registers and memory locations
pub mod mov;

/// # Comparison
/// Module for making logical comparison
pub mod cmp;

/// # Bit manipulation
/// Module for logical bit manipulation
pub mod bitman;

pub type PasmResult = Result<(), PasmError>;

#[derive(Debug)]
pub enum PasmError {
    Str(String),
    InvalidUtf8Byte(u8),
    InvalidLiteral,
    InvalidOperand,
    NoOperand,
    InvalidMemoryLoc(String),
    InvalidIndirectAddress,
}

impl Display for PasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PasmError::*;

        match self {
            Str(s) => f.write_str(s),
            InvalidUtf8Byte(b) => f.write_fmt(format_args!("The value in the ACC, `{}`, is not a valid UTF-8 byte.", b)),
            InvalidLiteral => f.write_str("Operand is not a decimal, hexadecimal, or binary number."),
            InvalidOperand => f.write_str("Operand is not an integer. If you wanted to use a label, please double-check the label."),
            NoOperand => f.write_str("Operand missing."),
            InvalidMemoryLoc(l) => f.write_fmt(format_args!("Memory location `{}` does not exist.", l)),
            InvalidIndirectAddress => f.write_str("The value at this memory location is not a valid memory location. If you wanted to use a label, please double-check the label."),
        }
    }
}

impl std::error::Error for PasmError {}

impl<T: Deref<Target = str>> From<T> for PasmError {
    fn from(s: T) -> Self {
        PasmError::Str(s.to_string())
    }
}

#[repr(transparent)]
pub struct Source(Vec<String>);

impl Source {
    fn handle_err(&self, err: &PasmError, pos: usize) {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Program {\n")?;

        for inst in &self.0 {
            f.write_fmt(format_args!("\t{}\n", inst))?;
        }

        f.write_str("}\n")
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Memory<K: Ord, V: Clone>(BTreeMap<K, V>);

impl<K: Ord + Debug, V: Clone> Memory<K, V> {
    #[must_use]
    pub fn new(data: BTreeMap<K, V>) -> Memory<K, V> {
        Memory(data)
    }

    pub fn get(&self, loc: &K) -> Result<V, PasmError> {
        let x = self
            .0
            .get(loc)
            .ok_or_else(|| PasmError::InvalidMemoryLoc(format!("{:?}", loc)))?;
        Ok(x.clone())
    }

    pub fn write(&mut self, loc: &K, dat: V) -> PasmResult {
        let x = self
            .0
            .get_mut(loc)
            .ok_or_else(|| PasmError::InvalidMemoryLoc(format!("{:?}", loc)))?;
        *x = dat;

        Ok(())
    }
}

pub type Op = Option<String>;

pub type Func = fn(&mut Context, Op) -> PasmResult;

pub type Cmd = (Func, Op);

pub struct Context {
    pub cmpr: bool,
    pub mar: usize,
    pub acc: usize,
    pub ix: usize,
    pub flow_override_reg: bool,
    pub mem: Memory<usize, usize>,
    pub add_regs: Vec<usize>,
}

impl Context {
    #[must_use]
    pub fn new(mem: Memory<usize, usize>, add_regs: Option<Vec<usize>>) -> Context {
        Context {
            cmpr: false,
            mar: 0,
            acc: 0,
            ix: 0,
            flow_override_reg: false,
            mem,
            add_regs: add_regs.map_or(vec![], |regs| regs),
        }
    }

    #[inline]
    pub fn override_flow_control(&mut self) {
        self.flow_override_reg = true;
    }

    #[inline]
    pub fn increment(&mut self) -> PasmResult {
        self.mar += 1;
        self.flow_override_reg = true;
        Ok(())
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("mar", &self.mar)
            .field("acc", &self.acc)
            .field("ix", &self.ix)
            .field("cmpr", &self.cmpr)
            .field("mem", &self.mem)
            .finish()
    }
}

pub struct Executor {
    source: Source,
    prog: Memory<usize, Cmd>,
    pub(crate) ctx: Context,
    count: u64,
}

impl Executor {
    #[must_use]
    pub fn new(source: impl Into<Source>, prog: Memory<usize, Cmd>, ctx: Context) -> Executor {
        Executor {
            source: source.into(),
            prog,
            ctx,
            count: 0,
        }
    }

    pub fn exec(&mut self) {
        loop {
            if self.ctx.mar == self.prog.0.len() {
                break;
            }

            self.count += 1;

            trace!("Executing line {}", self.ctx.mar + 1);

            let cir = if let Ok(cir) = self.prog.get(&self.ctx.mar) {
                cir
            } else {
                panic!("Unable to fetch instruction. Please report this as a bug with full debug logs attached.")
            };

            match cir.0(&mut self.ctx, cir.1) {
                Ok(_) => (),
                Err(e) => {
                    self.source.handle_err(&e, self.ctx.mar);
                    return;
                }
            }

            if self.ctx.flow_override_reg {
                self.ctx.flow_override_reg = false;
            } else {
                self.ctx.mar += 1;
            }
        }

        debug!("Total instructions executed: {}", self.count);
    }
}

impl Debug for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.prog.0.iter().map(|(line, (_, op))| {
                (
                    line,
                    if op.is_some() {
                        op.as_ref().unwrap()
                    } else {
                        ""
                    },
                )
            }))
            .finish()
    }
}

/// Macro to generate an instruction implementation
///
/// # Examples
/// ```
/// // Ensure all types are imported
/// use cambridge_asm::{exec::{PasmResult, Op, Context}, inst};
///
/// // No Context
/// inst!(name1 { /* Do something that doesn't need context or op*/ });
///
/// // Context only
/// inst!(name2 | ctx | { /* Do something with ctx */ });
///
/// // Context and op
/// inst!(name3 | ctx, op | { /* Do something with ctx and op */ });
/// ```
///
/// For further reference, look at the source of the module [`exec::io`]
#[macro_export]
macro_rules! inst {
    ($(#[$outer:meta])* $name:ident |$ctx:ident, $op:ident| { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, $op: $crate::exec::Op) -> $crate::exec::PasmResult {
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident |$ctx:ident| { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, _: $crate::exec::Op) -> $crate::exec::PasmResult {
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name(_: &mut $crate::exec::Context, _: $crate::exec::Op) -> $crate::exec::PasmResult {
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident |$ctx:ident, $op:ident| override { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, $op: $crate::exec::Op) -> $crate::exec::PasmResult {
            $ctx.override_flow_control();
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident |$ctx:ident| override { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, _: $crate::exec::Op) -> $crate::exec::PasmResult {
            $ctx.override_flow_control();
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident override { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name(ctx: &mut $crate::exec::Context, _: $crate::exec::Op) -> $crate::exec::PasmResult {
            ctx.override_flow_control();
            $( $code )*
            Ok(())
        }
    };
}

#[cfg(test)]
#[test]
fn exec() {
    let mut prog: BTreeMap<usize, Cmd> = BTreeMap::new();
    let mut mem: BTreeMap<usize, usize> = BTreeMap::new();

    // Division algorithm from pg 101 of textbook
    prog.insert(0, (mov::ldd, Some("200".into())));
    prog.insert(1, (mov::sto, Some("202".into())));
    prog.insert(2, (mov::sto, Some("203".into())));
    prog.insert(3, (mov::ldd, Some("202".into())));
    prog.insert(4, (arith::inc, Some("ACC".into())));
    prog.insert(5, (mov::sto, Some("202".into())));
    prog.insert(6, (mov::ldd, Some("203".into())));
    prog.insert(7, (arith::add, Some("201".into())));
    prog.insert(8, (mov::sto, Some("203".into())));
    prog.insert(9, (cmp::cmp, Some("204".into())));
    prog.insert(10, (cmp::jpn, Some("3".into())));
    prog.insert(11, (mov::ldd, Some("202".into())));
    prog.insert(12, (io::out, None));
    prog.insert(13, (io::end, None));

    // Memory partition
    mem.insert(200, 0);
    mem.insert(201, 5);
    mem.insert(202, 0);
    mem.insert(203, 0);
    mem.insert(204, 75);

    let mut exec = Executor {
        source: "None".into(),
        prog: Memory::new(prog),
        ctx: Context::new(Memory::new(mem), None),
        count: 0,
    };

    exec.exec();

    assert_eq!(exec.ctx.acc, 15);
}
