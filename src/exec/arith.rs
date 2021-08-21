// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};

pub fn add(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;

    let y = ctx.mem.get(&x)?;

    if let Some(res) = ctx.acc.checked_add(y) {
        ctx.acc = res;
    } else {
        warn!("Addition overflow detected at line {}", ctx.mar + 1);
        ctx.acc += y;
    }

    Ok(())
}

pub fn addm(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidLiteral)?;

    if let Some(res) = ctx.acc.checked_add(x) {
        ctx.acc = res;
    } else {
        warn!("Addition overflow detected at line {}", ctx.mar + 1);
        ctx.acc += x;
    }

    Ok(())
}

pub fn sub(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;

    let y = ctx.mem.get(&x)?;

    if let Some(res) = ctx.acc.checked_sub(y) {
        ctx.acc = res;
    } else {
        warn!("Subtraction overflow detected at line {}", ctx.mar + 1);
        ctx.acc -= y;
    }

    Ok(())
}

pub fn subm(ctx: &mut Context, op: Op) -> PasmResult {
    let x: usize = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidLiteral)?;

    if let Some(res) = ctx.acc.checked_sub(x) {
        ctx.acc = res;
    } else {
        warn!("Subtraction overflow detected at line {}", ctx.mar + 1);
        ctx.acc -= x;
    }

    Ok(())
}

pub fn inc(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op.ok_or(PasmError::NoOperand)?;

    match x.as_str() {
        "ix" | "IX" => ctx.ix += 1,
        "acc" | "ACC" => ctx.acc += 1,
        _ => return Err(PasmError::from("Only 'IX' and 'ACC' are valid registers")),
    }

    Ok(())
}

pub fn dec(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op.ok_or(PasmError::NoOperand)?;

    match x.as_str() {
        "ix" | "IX" => ctx.ix -= 1,
        "acc" | "ACC" => ctx.acc -= 1,
        _ => return Err(PasmError::from("Only 'IX' and 'ACC' are valid registers")),
    }

    Ok(())
}
