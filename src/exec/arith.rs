// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};

pub fn add(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;

    ctx.acc += ctx.mem.get(&x)?;

    ctx.increment()
}

pub fn addm(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| PasmError::from("Operand is not a decimal, hexadecimal, or binary number."))?;

    ctx.acc += x;

    ctx.increment()
}

pub fn sub(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;

    ctx.acc -= ctx.mem.get(&x)?;

    ctx.increment()
}

pub fn subm(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| PasmError::from("Operand is not a decimal, hexadecimal, or binary number."))?;

    ctx.acc -= x;

    ctx.increment()
}

pub fn inc(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op.ok_or_else(|| PasmError::from("No Operand"))?;

    match x.as_str() {
        "ix" | "IX" => ctx.ix += 1,
        "acc" | "ACC" => ctx.acc += 1,
        _ => return Err(PasmError::from("Only 'IX' and 'ACC' are valid registers")),
    }

    ctx.increment()
}

pub fn dec(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op.ok_or_else(|| PasmError::from("No Operand"))?;

    match x.as_str() {
        "ix" | "IX" => ctx.ix -= 1,
        "acc" | "ACC" => ctx.acc -= 1,
        _ => return Err(PasmError::from("Only 'IX' and 'ACC' are valid registers")),
    }

    ctx.increment()
}
