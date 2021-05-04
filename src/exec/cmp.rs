// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};

pub fn jmp(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;

    ctx.mar = x;

    Ok(())
}

pub fn cmp(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;

    ctx.cmpr = ctx.acc == ctx.mem.get(&x)?;

    ctx.increment()
}

pub fn cmpm(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| PasmError::from("Operand is not a decimal, hexadecimal, or binary number."))?;

    ctx.cmpr = ctx.acc == x;

    ctx.increment()
}

pub fn cmi(ctx: &mut Context, op: Op) -> PasmResult {
    let mut x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;

    x = ctx.mem.get(&x)?;

    ctx.cmpr = ctx.acc == ctx.mem.get(&x).map_err(|_| PasmError::from("The value at this memory location is not a valid memory location. Did you want to use a label? If so, check the label."))?;

    ctx.increment()
}

pub fn jpe(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;

    if ctx.cmpr {
        ctx.mar = x;
        Ok(())
    } else {
        ctx.increment()
    }
}

pub fn jpn(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;

    if ctx.cmpr {
        ctx.increment()
    } else {
        ctx.mar = x;
        Ok(())
    }
}
