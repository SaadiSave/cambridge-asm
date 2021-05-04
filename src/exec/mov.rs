// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};

pub fn ldm(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| PasmError::from("Operand is not a decimal, hexadecimal, or binary number."))?;

    ctx.acc = x;

    ctx.increment()
}

pub fn ldd(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;

    ctx.acc = ctx.mem.get(&x)?;

    ctx.increment()
}

pub fn ldi(ctx: &mut Context, op: Op) -> PasmResult {
    let mut x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;

    x = ctx.mem.get(&x)?;

    ctx.acc = ctx.mem.get(&x).map_err(|_| PasmError::from("The value at this memory location is not a valid memory location. Did you want to use a label? If so, check the label."))?;

    ctx.increment()
}

pub fn ldx(ctx: &mut Context, op: Op) -> PasmResult {
    let mut x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;
    x += ctx.ix;

    ctx.acc = ctx.mem.get(&x)?;

    ctx.increment()
}

pub fn ldr(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| PasmError::from("Operand is not a decimal, hexadecimal, or binary number."))?;

    ctx.ix = x;

    ctx.increment()
}

pub fn mov(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op.ok_or_else(|| PasmError::from("No Operand"))?;

    match x.as_str() {
        "ix" | "IX" => ctx.ix = ctx.acc,
        _ => {
            return Err(PasmError::from(
                "Only 'IX' is a valid operand for this instruction",
            ))
        }
    }

    ctx.increment()
}

pub fn sto(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;

    ctx.mem.write(&x, ctx.acc)?;

    ctx.increment()
}
