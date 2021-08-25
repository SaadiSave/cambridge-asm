// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};

pub fn ldm(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidLiteral)?;

    ctx.acc = x;

    Ok(())
}

pub fn ldd(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;

    ctx.acc = ctx.mem.get(&x)?;

    Ok(())
}

pub fn ldi(ctx: &mut Context, op: Op) -> PasmResult {
    let mut x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;

    x = ctx.mem.get_address(&x)?;

    ctx.acc = ctx
        .mem
        .get(&x)
        .map_err(|_| PasmError::InvalidIndirectAddress(x))?;

    Ok(())
}

pub fn ldx(ctx: &mut Context, op: Op) -> PasmResult {
    let mut x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;
    x += ctx.ix;

    ctx.acc = ctx.mem.get(&x)?;

    Ok(())
}

pub fn ldr(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidLiteral)?;

    ctx.ix = x;

    Ok(())
}

pub fn mov(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op.ok_or(PasmError::NoOperand)?;

    match x.as_str() {
        "ix" | "IX" => ctx.ix = ctx.acc,
        _ => {
            return Err(PasmError::from(
                "Only 'IX' is a valid operand for this instruction",
            ))
        }
    }

    Ok(())
}

pub fn sto(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;

    ctx.mem.write(&x, ctx.acc)?;

    Ok(())
}
