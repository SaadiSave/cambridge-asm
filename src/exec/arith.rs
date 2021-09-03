// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};

pub fn add(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(x) => {
            let y = ctx.mem.get(&x)?;

            if let Some(res) = ctx.acc.checked_add(y) {
                ctx.acc = res;
            } else {
                warn!("Addition overflow detected at line {}", ctx.mar + 1);
                ctx.acc += y;
            }
        }
        Op::Literal(x) => {
            if let Some(res) = ctx.acc.checked_add(x) {
                ctx.acc = res;
            } else {
                warn!("Addition overflow detected at line {}", ctx.mar + 1);
                ctx.acc += x;
            }
        }
        Op::None => return Err(PasmError::NoOperand),
        _ => return Err(PasmError::InvalidOperand),
    }

    Ok(())
}

pub fn sub(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(x) => {
            let y = ctx.mem.get(&x)?;

            if let Some(res) = ctx.acc.checked_sub(y) {
                ctx.acc = res;
            } else {
                warn!("Addition overflow detected at line {}", ctx.mar + 1);
                ctx.acc -= y;
            }
        }
        Op::Literal(x) => {
            if let Some(res) = ctx.acc.checked_sub(x) {
                ctx.acc = res;
            } else {
                warn!("Addition overflow detected at line {}", ctx.mar + 1);
                ctx.acc -= x;
            }
        }
        Op::None => return Err(PasmError::NoOperand),
        _ => return Err(PasmError::InvalidOperand),
    }

    Ok(())
}

pub fn inc(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Acc => ctx.acc += 1,
        Op::Ix => ctx.ix += 1,
        Op::None => return Err(PasmError::NoOperand),
        _ => return Err(PasmError::InvalidOperand),
    }

    Ok(())
}

pub fn dec(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Acc => ctx.acc -= 1,
        Op::Ix => ctx.ix -= 1,
        Op::None => return Err(PasmError::NoOperand),
        _ => return Err(PasmError::InvalidOperand),
    }

    Ok(())
}
