// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};

pub fn and(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;

    ctx.acc &= ctx.mem.get(&x)?;

    Ok(())
}

pub fn andm(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidLiteral)?;

    ctx.acc &= x;

    Ok(())
}

pub fn or(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;

    ctx.acc |= ctx.mem.get(&x)?;

    Ok(())
}

pub fn orm(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidLiteral)?;

    ctx.acc |= x;

    Ok(())
}

pub fn xor(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;

    ctx.acc ^= ctx.mem.get(&x)?;

    Ok(())
}

pub fn xorm(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidLiteral)?;

    ctx.acc ^= x;

    Ok(())
}

pub fn lsl(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidLiteral)?;

    ctx.acc <<= x;

    Ok(())
}

pub fn lsr(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidLiteral)?;

    ctx.acc >>= x;

    Ok(())
}
