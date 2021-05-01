// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{collections::BTreeMap, fmt::Debug};

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

#[derive(Debug)]
pub struct Memory<K: std::fmt::Display + Ord, V: Clone>(pub BTreeMap<K, V>);

impl<K: std::fmt::Display + Ord, V: Clone> Memory<K, V> {
    pub fn get(&self, loc: &K) -> V {
        let x = self
            .0
            .get(loc)
            .unwrap_or_else(|| panic!("Memory does not contain address {}", loc));
        x.clone()
    }

    pub fn write(&mut self, loc: &K, dat: V) {
        let x = self
            .0
            .get_mut(loc)
            .unwrap_or_else(|| panic!("Memory does not contain address {}", loc));
        *x = dat;
    }
}

pub type Op = Option<String>;

pub type Func = fn(&mut Context, Op);

pub type Cmd = (Func, Op);

#[derive(Debug)]
pub struct Context {
    pub cmpr: bool,
    pub mar: usize,
    pub acc: usize,
    pub ix: usize,
    pub mem: Memory<usize, usize>,
}

impl Context {
    pub fn increment(&mut self) {
        self.mar += 1
    }
}

pub struct Executor {
    pub prog: Memory<usize, Cmd>,
    pub ctx: Context,
}

impl Executor {
    pub fn exec(&mut self) {
        loop {
            if self.ctx.mar == self.prog.0.len() {
                break;
            }
            let cir = self.prog.get(&self.ctx.mar);
            cir.0(&mut self.ctx, cir.1)
        }
    }
}

impl Debug for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Executor {\nprog: {\n").unwrap();
        for i in self.prog.0.iter() {
            f.write_fmt(format_args!("{:?}\n", (i.0, (i.1).1.as_ref()))).unwrap();
        };
        f.write_str("}\n").unwrap();
        f.write_fmt(format_args!("{:?}", self.ctx))
    }
}

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
    prog.insert(12, (io::dbg, Some("ACC".into())));
    prog.insert(13, (io::end, None));

    // Memory partition
    mem.insert(200, 0);
    mem.insert(201, 5);
    mem.insert(202, 0);
    mem.insert(203, 0);
    mem.insert(204, 75);


    let mut exec = Executor {
        prog: Memory(prog),
        ctx: Context {
            cmpr: false,
            mar: 0,
            acc: 0,
            ix: 0,
            mem: Memory(mem),
        },
    };
    
    let t = std::time::Instant::now();
    exec.exec();
    println!("{:?}", t.elapsed())
}
