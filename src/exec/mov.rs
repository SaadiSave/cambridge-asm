// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};

pub fn ldm(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Literal(x) => {
            ctx.acc = x;
            Ok(())
        }
        Op::None => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidLiteral),
    }
}

pub fn ldd(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(x) => {
            ctx.acc = ctx.mem.get(&x)?;
            Ok(())
        }
        Op::None => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}

pub fn ldi(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(mut x) => {
            x = ctx.mem.get_address(&x)?;

            ctx.acc = ctx
                .mem
                .get(&x)
                .map_err(|_| PasmError::InvalidIndirectAddress(x))?;

            Ok(())
        }
        Op::None => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}

pub fn ldx(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(mut x) => {
            x += ctx.ix;

            ctx.acc = ctx.mem.get(&x)?;

            Ok(())
        }
        Op::None => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}

pub fn ldr(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Literal(x) => {
            ctx.ix = x;
        }
        Op::Loc(x) => {
            ctx.ix = ctx.mem.get(&x)?;
        }
        Op::None => return Err(PasmError::NoOperand),
        _ => return Err(PasmError::InvalidOperand),
    }

    Ok(())
}

pub fn mov(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Ix => ctx.ix = ctx.acc,
        Op::Loc(x) => ctx.mem.write(&x, ctx.ix)?,
        Op::MultiOp(list) => match list[..] {
            [Op::Loc(from), Op::Loc(to), ..] => {
                let x = ctx.mem.get(&from)?;
                ctx.mem.write(&to, x)?;
            }
            _ => return Err(PasmError::InvalidMultiOp),
        },
        Op::None => return Err(PasmError::NoOperand),
        _ => return Err(PasmError::InvalidOperand),
    }

    Ok(())
}

pub fn sto(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(x) => {
            ctx.mem.write(&x, ctx.acc)?;

            Ok(())
        }
        Op::None => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}
