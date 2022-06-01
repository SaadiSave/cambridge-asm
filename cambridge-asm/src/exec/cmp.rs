// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, PasmError::*, PasmResult};
use crate::inst::Op::{self, *};

/// Jump
///
/// # Syntax
/// 1. `JMP [ref]` - jump to addr
/// 2. `JMP [ref],[ref]` - jump to first if CMP true, second if CMP false
pub fn jmp(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        &Addr(x) => {
            ctx.override_flow_control();
            ctx.mar = x;
        }
        MultiOp(ops) => match ops[..] {
            [Addr(eq), Addr(ne)] => {
                ctx.override_flow_control();
                if ctx.cmp {
                    ctx.mar = eq;
                } else {
                    ctx.mar = ne;
                }
            }
            _ => return Err(InvalidMultiOp),
        },
        Null => return Err(NoOperand),
        _ => return Err(InvalidOperand),
    }

    Ok(())
}

/// Compare
///
/// # Syntax
/// 1. `CMP [lit | reg | addr]` - compare to ACC
/// 2. `CMP [lit | reg | addr],[lit | reg | addr]` - compare both values
pub fn cmp(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        MultiOp(ops) => match ops[..] {
            [ref a, ref b] if a.is_usizeable() && b.is_usizeable() => {
                ctx.cmp = a.get_val(ctx)? == b.get_val(ctx)?;
            }
            _ => return Err(InvalidMultiOp),
        },
        val if val.is_usizeable() => ctx.cmp = ctx.acc == val.get_val(ctx)?,
        Null => return Err(NoOperand),
        _ => return Err(InvalidOperand),
    }

    Ok(())
}

/// Compare with indirect addressing
///
/// # Syntax
/// 1. `CMI [addr]`
/// 2. `CMI [lit | reg | addr],[addr]`
pub fn cmi(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        &Addr(addr) => {
            let addr2 = ctx.mem.get_address(&addr)?;

            ctx.cmp = ctx.acc
                == ctx
                    .mem
                    .get(&addr2)
                    .map_err(|_| InvalidIndirectAddress(addr))?;

            Ok(())
        }
        MultiOp(ops) => match ops[..] {
            [ref dest, Addr(addr)] if dest.is_usizeable() => {
                let addr2 = ctx.mem.get_address(&addr)?;

                ctx.cmp = dest.get_val(ctx)?
                    == ctx
                        .mem
                        .get(&addr2)
                        .map_err(|_| InvalidIndirectAddress(addr))?;

                Ok(())
            }
            _ => Err(InvalidMultiOp),
        },
        Null => Err(NoOperand),
        _ => Err(InvalidOperand),
    }
}

/// Jump if equal
///
/// # Syntax
/// `JPE [addr]`
pub fn jpe(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        &Addr(addr) => {
            if ctx.cmp {
                ctx.override_flow_control();
                ctx.mar = addr;
            }

            Ok(())
        }
        Null => Err(NoOperand),
        _ => Err(InvalidOperand),
    }
}

/// Jump if not equal
///
/// # Syntax
/// `JPN [addr]`
pub fn jpn(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        &Addr(addr) => {
            if !ctx.cmp {
                ctx.override_flow_control();
                ctx.mar = addr;
            }

            Ok(())
        }
        Null => Err(NoOperand),
        _ => Err(InvalidOperand),
    }
}
