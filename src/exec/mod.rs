// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    collections::BTreeMap,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

/// # Arithmetic
/// Module for arithmetic operations
#[allow(clippy::needless_pass_by_value, clippy::enum_glob_use)]
pub mod arith;

/// # I/O
/// Module for input, output and debugging
#[allow(clippy::needless_pass_by_value, clippy::enum_glob_use)]
pub mod io;

/// # Data movement
/// Module for moving data between registers and memory locations
#[allow(clippy::needless_pass_by_value, clippy::enum_glob_use)]
pub mod mov;

/// # Comparison
/// Module for making logical comparisons
#[allow(clippy::needless_pass_by_value, clippy::enum_glob_use)]
pub mod cmp;

/// # Bit manipulation
/// Module for logical bit manipulation
#[allow(clippy::needless_pass_by_value, clippy::enum_glob_use)]
pub mod bitman;

#[allow(clippy::enum_glob_use)]
mod error;

mod memory;

#[allow(clippy::enum_glob_use)]
mod inst;

pub use error::{PasmError, PasmResult, Source};

pub use memory::{MemEntry, Memory};

pub use inst::{Inst, Op, OpFun};

#[derive(Debug, Default)]
pub struct Context {
    pub cmp: bool,
    pub mar: usize,
    pub acc: usize,
    pub ix: usize,
    pub flow_override_reg: bool,
    pub mem: Memory,
    pub ret: usize,
    pub gprs: [usize; 30],
    pub end: bool,
}

impl Context {
    pub fn new(mem: Memory) -> Context {
        Context {
            mem,
            ..Context::default()
        }
    }

    #[inline]
    pub fn override_flow_control(&mut self) {
        self.flow_override_reg = true;
    }

    /// # Panics
    /// If `op` is not a `usize` register. To avoid this, check `op` using [`Op::is_register`].
    #[inline]
    pub fn get_mut_register(&mut self, op: &Op) -> &mut usize {
        match op {
            Op::Acc => &mut self.acc,
            Op::Ix => &mut self.ix,
            Op::Ar => &mut self.ret,
            Op::Gpr(x) => &mut self.gprs[*x],
            _ => unreachable!(),
        }
    }

    /// # Panics
    /// If `op` is not a `usize` register. To avoid this, check `op` using [`Op::is_register`].
    #[inline]
    pub fn get_register(&self, op: &Op) -> usize {
        match op {
            Op::Acc => self.acc,
            Op::Ix => self.ix,
            Op::Ar => self.ret,
            Op::Gpr(x) => self.gprs[*x],
            _ => unreachable!(),
        }
    }

    /// # Panics
    /// If `op` is not writable. To avoid this, check `op` using [`Op::is_read_write`].
    pub fn modify(&mut self, op: &Op, f: impl Fn(&mut usize)) -> PasmResult {
        match op {
            Op::Loc(x) => {
                let mut res = self.mem.get(x)?;
                f(&mut res);
                self.mem.write(x, res)?;
            }
            op if op.is_register() => f(self.get_mut_register(op)),
            _ => unreachable!(),
        }

        Ok(())
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Context {\n")?;
        f.write_fmt(format_args!("{:>6}: {}\n", "mar", self.mar))?;
        f.write_fmt(format_args!("{:>6}: {}\n", "acc", self.acc))?;
        f.write_fmt(format_args!("{:>6}: {}\n", "ix", self.ix))?;
        f.write_fmt(format_args!("{:>6}: {}\n", "cmp", self.cmp))?;
        f.write_fmt(format_args!(
            "{:>6}: {}\n",
            "gprs",
            self.gprs
                .iter()
                .enumerate()
                .fold(String::from("["), |s, (num, val)| {
                    if num == self.gprs.len() - 1 {
                        format!("{s}r{num} = {val}]")
                    } else {
                        format!("{s}r{num} = {val}, ")
                    }
                })
        ))?;
        f.write_fmt(format_args!("{:>6}: Memory {{\n", "mem"))?;

        for (addr, entry) in self.mem.iter() {
            f.write_fmt(format_args!("{addr:>8}: {entry},\n"))?;
        }

        f.write_fmt(format_args!("{:>3}}}\n", ""))?;

        f.write_str("}")
    }
}

pub type ExTree = BTreeMap<usize, Inst>;

pub struct Executor {
    pub source: Source,
    pub prog: ExTree,
    pub ctx: Context,
    count: u64,
}

impl Executor {
    pub fn new(source: impl Into<Source>, prog: ExTree, ctx: Context) -> Executor {
        Executor {
            source: source.into(),
            prog,
            ctx,
            count: 0,
        }
    }

    pub fn exec(&mut self) {
        loop {
            if self.ctx.mar == self.prog.len() || self.ctx.end {
                break;
            }

            self.count += 1;

            trace!("Executing line {}", self.ctx.mar + 1);

            let cir = if let Some(cir) = self.prog.get(&self.ctx.mar) {
                cir
            } else {
                panic!("Unable to fetch instruction. Please report this as a bug with full debug logs attached.")
            };

            match (cir.opfun)(&mut self.ctx, &cir.op) {
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

impl Display for Executor {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Executor {\n")?;
        for (addr, Inst { op, .. }) in &self.prog {
            f.write_fmt(format_args!("{addr:>6}: {op},\n", op = op.to_string()))?;
        }
        f.write_str("}")
    }
}

impl Debug for Executor {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Executor")
            .field("source", &self.source)
            .field(
                "prog",
                &self
                    .prog
                    .iter()
                    .map(|(addr, Inst { op, .. })| (addr, op))
                    .collect::<Vec<_>>(),
            )
            .field("ctx", &self.ctx)
            .field("count", &self.count)
            .finish()
    }
}

#[cfg(test)]
#[test]
fn exec() {
    use std::collections::BTreeMap;

    let prog: BTreeMap<usize, Inst> = BTreeMap::from(
        // Division algorithm from pg 101 of textbook
        [
            (0, Inst::new(mov::ldd, "200".into())),
            (1, Inst::new(mov::sto, "202".into())),
            (2, Inst::new(mov::sto, "203".into())),
            (3, Inst::new(mov::ldd, "202".into())),
            (4, Inst::new(arith::inc, "ACC".into())),
            (5, Inst::new(mov::sto, "202".into())),
            (6, Inst::new(mov::ldd, "203".into())),
            (7, Inst::new(arith::add, "201".into())),
            (8, Inst::new(mov::sto, "203".into())),
            (9, Inst::new(cmp::cmp, "204".into())),
            (10, Inst::new(cmp::jpn, "3".into())),
            (11, Inst::new(mov::ldd, "202".into())),
            (12, Inst::new(io::out, "".into())),
            (13, Inst::new(io::end, "".into())),
        ],
    );

    let mem: BTreeMap<usize, MemEntry> = BTreeMap::from([
        (200, 0.into()),
        (201, 5.into()),
        (202, 0.into()),
        (203, 0.into()),
        (204, 75.into()),
    ]);

    let mut exec = Executor::new("None", prog, Context::new(Memory::new(mem)));

    exec.exec();

    assert_eq!(exec.ctx.acc, 15);
}
