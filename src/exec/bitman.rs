// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};

pub fn and(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(x) => ctx.acc &= ctx.mem.get(&x)?,
        Op::Literal(x) => ctx.acc &= x,
        Op::None => return Err(PasmError::NoOperand),
        _ => return Err(PasmError::InvalidOperand),
    }

    Ok(())
}

pub fn or(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(x) => ctx.acc |= ctx.mem.get(&x)?,
        Op::Literal(x) => ctx.acc |= x,
        Op::None => return Err(PasmError::NoOperand),
        _ => return Err(PasmError::InvalidOperand),
    }

    Ok(())
}

pub fn xor(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(x) => ctx.acc ^= ctx.mem.get(&x)?,
        Op::Literal(x) => ctx.acc ^= x,
        Op::None => return Err(PasmError::NoOperand),
        _ => return Err(PasmError::InvalidOperand),
    }

    Ok(())
}

pub fn lsl(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Literal(x) => {
            if let Some(res) = ctx.acc.checked_shl(x as u32) {
                ctx.acc = res;
            } else {
                warn!("Shift left overflow detected at line {}", ctx.mar + 1);
                ctx.acc <<= x;
            }

            Ok(())
        }
        Op::None => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}

pub fn lsr(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Literal(x) => {
            ctx.acc >>= x;

            Ok(())
        }
        Op::None => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}
