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
#[repr(transparent)]
pub struct PasmError(String);

impl Display for PasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for PasmError {}

impl<T: Deref<Target = str>> From<T> for PasmError {
    fn from(s: T) -> Self {
        PasmError(s.to_string())
    }
}

#[repr(transparent)]
pub struct Source(Vec<String>);

impl Source {
    fn handle_err(&self, err: &PasmError, pos: usize) -> ! {
        let mut out = String::new();
        out.push_str("Error {\n");

        for (i, s) in self.0.iter().enumerate() {
            if pos == i {
                out.push_str(&format!("\n    {}\t{}", i + 1, s));
                out.push_str(&format!("\t< {}\n", &err.0));
                out.push('\n');
                break;
            }
        }

        out.push('}');
        panic!("{}", &out);
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
            f.write_fmt(format_args!("\t{}\n", &inst))?;
        }

        f.write_str("}\n")
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Memory<K: Ord, V: Clone>(pub BTreeMap<K, V>);

impl<K: Ord, V: Clone> Memory<K, V> {
    pub fn get(&self, loc: &K) -> Result<V, PasmError> {
        let x = self
            .0
            .get(loc)
            .ok_or_else(|| PasmError::from("Memory does not contain this location"))?;
        Ok(x.clone())
    }

    pub fn write(&mut self, loc: &K, dat: V) -> PasmResult {
        let x = self
            .0
            .get_mut(loc)
            .ok_or_else(|| PasmError::from("Memory does not contain this location"))?;
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
    pub mem: Memory<usize, usize>,
    pub add_regs: Vec<usize>,
}

impl Context {
    pub fn increment(&mut self) -> PasmResult {
        self.mar += 1;

        Ok(())
    }
}

impl Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Context {\n")?;
        f.write_fmt(format_args!("    mar: {}\n", &self.mar))?;
        f.write_fmt(format_args!("    acc: {}\n", &self.acc))?;
        f.write_fmt(format_args!("    ix: {}\n", &self.ix))?;
        f.write_fmt(format_args!("    cmpr: {}\n", &self.cmpr))?;
        f.write_fmt(format_args!("    mem: {:?}\n", &self.mem))?;
        f.write_str("}\n")
    }
}

pub struct Executor {
    pub source: Source,
    pub prog: Memory<usize, Cmd>,
    pub ctx: Context,
    pub count: u64,
}

impl Executor {
    pub fn exec(&mut self) {
        loop {
            if self.ctx.mar == self.prog.0.len() {
                break;
            }

            self.count += 1;

            trace!("Executing line {}", self.ctx.mar + 1);

            let cir = self.prog.get(&self.ctx.mar).unwrap_or_else(|_| {
                self.source.handle_err(
                    &PasmError::from("Unable to fetch instruction. Please report this as a bug."),
                    self.ctx.mar,
                )
            });
            cir.0(&mut self.ctx, cir.1).unwrap_or_else(|e| self.source.handle_err(&e, self.ctx.mar));
        }

        debug!("Total instructions executed: {}", self.count)
    }
}

impl Debug for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Executor {\n")?;

        for i in &self.prog.0 {
            f.write_fmt(format_args!("    {:?}\n", (i.0, (i.1).1.as_ref())))?;
        }

        f.write_str("}\n")
    }
}

#[cfg(test)]
#[test]
fn exec() {
    let mut prog: BTreeMap<usize, Cmd> = BTreeMap::new();
    let mut mem: BTreeMap<usize, usize> = BTreeMap::new();

    // Divison algorithm from pg 101 of textbook
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
    prog.insert(12, (io::out, Some("ACC".into())));
    prog.insert(13, (io::end, None));

    // Memory partition
    mem.insert(200, 0);
    mem.insert(201, 5);
    mem.insert(202, 0);
    mem.insert(203, 0);
    mem.insert(204, 75);

    let mut exec = Executor {
        source: "None".into(),
        prog: Memory(prog),
        ctx: Context {
            cmpr: false,
            mar: 0,
            acc: 0,
            ix: 0,
            mem: Memory(mem),
            add_regs: vec![],
        },
        count: 0,
    };

    let t = std::time::Instant::now();
    exec.exec();
    println!("{:?}", t.elapsed())
}
