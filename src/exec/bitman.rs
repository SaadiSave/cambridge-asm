// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};

pub fn and(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;

    ctx.acc &= ctx.mem.get(&x)?;

    ctx.increment()
}

pub fn andm(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| PasmError::from("Operand is not a decimal, hexadecimal, or binary number."))?;

    ctx.acc &= x;

    ctx.increment()
}

pub fn or(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;

    ctx.acc |= ctx.mem.get(&x)?;

    ctx.increment()
}

pub fn orm(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| PasmError::from("Operand is not a decimal, hexadecimal, or binary number."))?;

    ctx.acc |= x;

    ctx.increment()
}

pub fn xor(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| {
            PasmError::from(
                "Operand is not an integer. Did you want to use a label? If so, check the label.",
            )
        })?;

    ctx.acc ^= ctx.mem.get(&x)?;

    ctx.increment()
}

pub fn xorm(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| PasmError::from("Operand is not a decimal, hexadecimal, or binary number."))?;

    ctx.acc ^= x;

    ctx.increment()
}

pub fn lsl(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| PasmError::from("Operand is not a decimal, hexadecimal, or binary number."))?;

    ctx.acc <<= x;

    ctx.increment()
}

pub fn lsr(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or_else(|| PasmError::from("No Operand"))?
        .parse()
        .map_err(|_| PasmError::from("Operand is not a decimal, hexadecimal, or binary number."))?;

    ctx.acc >>= x;

    ctx.increment()
}
