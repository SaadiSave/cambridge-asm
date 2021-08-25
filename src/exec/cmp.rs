// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};

pub fn jmp(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;

    ctx.override_flow_control();
    ctx.mar = x;

    Ok(())
}

pub fn cmp(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;

    ctx.cmpr = ctx.acc == ctx.mem.get(&x)?;

    Ok(())
}

pub fn cmpm(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidLiteral)?;

    ctx.cmpr = ctx.acc == x;

    Ok(())
}

pub fn cmi(ctx: &mut Context, op: Op) -> PasmResult {
    let mut x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;

    x = ctx.mem.get_address(&x)?;

    ctx.cmpr = ctx.acc
        == ctx
            .mem
            .get(&x)
            .map_err(|_| PasmError::InvalidIndirectAddress(x))?;

    Ok(())
}

pub fn jpe(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;

    if ctx.cmpr {
        ctx.override_flow_control();
        ctx.mar = x;
    }

    Ok(())
}

pub fn jpn(ctx: &mut Context, op: Op) -> PasmResult {
    let x = op
        .ok_or(PasmError::NoOperand)?
        .parse()
        .map_err(|_| PasmError::InvalidOperand)?;

    if !ctx.cmpr {
        ctx.override_flow_control();
        ctx.mar = x;
    }

    Ok(())
}
