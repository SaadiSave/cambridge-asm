// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};
use std::io::Read;

pub fn end(ctx: &mut Context, _: Op) -> PasmResult {
    ctx.increment()
}

pub fn out(ctx: &mut Context, _: Op) -> PasmResult {
    let x = ctx.acc;

    if x > 127 {
        return Err(PasmError::from(format!(
            "The value in the ACC, `{}`, is not valid ASCII.",
            &x
        )));
    }

    let out = x as u8 as char;

    println!("{}", &out);

    ctx.increment()
}

pub fn inp(ctx: &mut Context, _: Op) -> PasmResult {
    let mut x = [0; 1];

    std::io::stdin()
        .read_exact(&mut x)
        .expect("Unable to read stdin");

    ctx.acc = x[0] as usize;

    ctx.increment()
}

// Custom instruction for debug logging
pub fn dbg(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op.ok_or_else(|| PasmError::from("No Operand"))?;

    let out = match x.as_str() {
        "ix" | "IX" => ctx.ix,
        "acc" | "ACC" => ctx.acc,
        _ => {
            if let Ok(s) = x.parse() {
                ctx.mem.get(&s)?
            } else {
                return Err(PasmError::from(format!(
                    "{} is not a register or a memory address",
                    &x
                )));
            }
        }
    };

    println!("{}", &out);

    ctx.increment()
}

// Raw input - directly input integers
pub fn rin(ctx: &mut Context, _: Op) -> PasmResult {
    let mut x = String::new();

    std::io::stdin()
        .read_line(&mut x)
        .expect("Unable to read stdin");

    x.ends_with('\n').then(|| x.pop());
    x.ends_with('\r').then(|| x.pop());

    ctx.acc = x
        .parse()
        .unwrap_or_else(|_| panic!("'{}' is not an integer", &x));

    ctx.increment()
}
