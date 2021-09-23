// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fmt::{Debug, Formatter, Result as FmtResult};

/// # Arithmetic
/// Module for arithmetic operations
#[allow(clippy::needless_pass_by_value)]
pub mod arith;

/// # I/O
/// Module for input, output and debugging
#[allow(clippy::needless_pass_by_value)]
pub mod io;

/// # Data movement
/// Module for moving data between registers and memory locations
#[allow(clippy::needless_pass_by_value)]
pub mod mov;

/// # Comparison
/// Module for making logical comparison
#[allow(clippy::needless_pass_by_value)]
pub mod cmp;

/// # Bit manipulation
/// Module for logical bit manipulation
#[allow(clippy::needless_pass_by_value)]
pub mod bitman;

mod error;

mod memory;

mod inst;

pub use error::{PasmError, PasmResult, Source};

pub use memory::{MemEntry, Memory};

pub use inst::{Cmd, Func, Op};

pub struct Context {
    pub cmp: bool,
    pub mar: usize,
    pub acc: usize,
    pub ix: usize,
    pub flow_override_reg: bool,
    pub mem: Memory<usize, MemEntry>,
    pub add_regs: Vec<usize>,
}

impl Context {
    pub fn new(mem: Memory<usize, MemEntry>, add_regs: Option<Vec<usize>>) -> Context {
        Context {
            cmp: false,
            mar: 0,
            acc: 0,
            ix: 0,
            flow_override_reg: false,
            mem,
            add_regs: add_regs.unwrap_or_default(),
        }
    }

    #[inline]
    pub fn override_flow_control(&mut self) {
        self.flow_override_reg = true;
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Context")
            .field("mar", &self.mar)
            .field("acc", &self.acc)
            .field("ix", &self.ix)
            .field("cmp", &self.cmp)
            .field("mem", &self.mem)
            .finish()
    }
}

pub struct Executor {
    pub source: Source,
    pub prog: Memory<usize, Cmd>,
    pub ctx: Context,
    count: u64,
}

impl Executor {
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
            if self.ctx.mar == self.prog.len() {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_map()
            .entries(self.prog.iter().map(|(line, (_, op))| {
                (
                    line,
                    if op.is_none() {
                        "".to_string()
                    } else {
                        op.to_string()
                    },
                )
            }))
            .finish()
    }
}

#[cfg(test)]
#[test]
fn exec() {
    use std::collections::BTreeMap;

    let mut prog: BTreeMap<usize, Cmd> = BTreeMap::new();
    let mut mem: BTreeMap<usize, MemEntry> = BTreeMap::new();

    // Division algorithm from pg 101 of textbook
    prog.insert(0, (mov::ldd, "200".into()));
    prog.insert(1, (mov::sto, "202".into()));
    prog.insert(2, (mov::sto, "203".into()));
    prog.insert(3, (mov::ldd, "202".into()));
    prog.insert(4, (arith::inc, "ACC".into()));
    prog.insert(5, (mov::sto, "202".into()));
    prog.insert(6, (mov::ldd, "203".into()));
    prog.insert(7, (arith::add, "201".into()));
    prog.insert(8, (mov::sto, "203".into()));
    prog.insert(9, (cmp::cmp, "204".into()));
    prog.insert(10, (cmp::jpn, "3".into()));
    prog.insert(11, (mov::ldd, "202".into()));
    prog.insert(12, (io::out, "".into()));
    prog.insert(13, (io::end, "".into()));

    // Memory partition
    mem.insert(200, 0.into());
    mem.insert(201, 5.into());
    mem.insert(202, 0.into());
    mem.insert(203, 0.into());
    mem.insert(204, 75.into());

    let mut exec = Executor::new(
        "None",
        Memory::new(prog),
        Context::new(Memory::new(mem), None),
    );

    exec.exec();

    assert_eq!(exec.ctx.acc, 15);
}
