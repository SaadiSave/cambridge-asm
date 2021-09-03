// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};

pub fn jmp(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(x) => {
            ctx.override_flow_control();
            ctx.mar = x;

            Ok(())
        }
        Op::None => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}

pub fn cmp(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(x) => ctx.cmp = ctx.acc == ctx.mem.get(&x)?,
        Op::Literal(x) => ctx.cmp = ctx.acc == x,
        Op::None => return Err(PasmError::NoOperand),
        _ => return Err(PasmError::InvalidOperand),
    }

    Ok(())
}

pub fn cmi(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(mut x) => {
            x = ctx.mem.get_address(&x)?;

            ctx.cmp = ctx.acc
                == ctx
                    .mem
                    .get(&x)
                    .map_err(|_| PasmError::InvalidIndirectAddress(x))?;

            Ok(())
        }
        Op::None => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}

pub fn jpe(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(x) => {
            if ctx.cmp {
                ctx.override_flow_control();
                ctx.mar = x;
            }

            Ok(())
        }
        Op::None => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}

pub fn jpn(ctx: &mut Context, op: Op) -> PasmResult {
    match op {
        Op::Loc(x) => {
            if !ctx.cmp {
                ctx.override_flow_control();
                ctx.mar = x;
            }

            Ok(())
        }
        Op::None => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}
