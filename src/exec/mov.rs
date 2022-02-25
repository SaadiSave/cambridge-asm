// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op, PasmError, PasmResult};

/// Load immediate values into a register
///
/// # Syntax
///
/// 1. `LDM [lit]` - loads to `ACC`
/// 2. `LDM [reg],[lit]` - loads to `reg`
pub fn ldm(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        Op::MultiOp(ops) => match ops[..] {
            [ref op, Op::Literal(val)] if op.is_register() => {
                *ctx.get_mut_register(op) = val;
                Ok(())
            }
            _ => Err(PasmError::InvalidMultiOp),
        },
        &Op::Literal(val) => {
            ctx.acc = val;
            Ok(())
        }
        Op::Null => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}

/// Load values from memory into a register
///
/// # Syntax
///
/// 1. `LDD [loc]` - loads to `ACC`
/// 2. `LDD [reg],[loc]` - loads to `reg`
pub fn ldd(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        Op::Loc(loc) => {
            ctx.acc = ctx.mem.get(loc)?;
            Ok(())
        }
        Op::MultiOp(ops) => match ops[..] {
            [ref reg, Op::Loc(ref loc)] if reg.is_register() => {
                let x = ctx.mem.get(loc)?;
                *ctx.get_mut_register(reg) = x;
                Ok(())
            }
            _ => Err(PasmError::InvalidMultiOp),
        },
        Op::Null => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}

/// Load values from memory using indirect addressing into a register
///
/// # Syntax
///
/// 1. `LDM [loc]` - loads to `ACC`
/// 2. `LDM [reg],[loc]` - loads to `reg`
pub fn ldi(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        Op::Loc(mut loc) => {
            loc = ctx.mem.get_address(&loc)?;

            ctx.acc = ctx
                .mem
                .get(&loc)
                .map_err(|_| PasmError::InvalidIndirectAddress(loc))?;

            Ok(())
        }
        Op::MultiOp(ops) => match ops[..] {
            [ref reg, Op::Loc(mut loc)] if reg.is_register() => {
                loc = ctx.mem.get_address(&loc)?;

                *ctx.get_mut_register(reg) = ctx
                    .mem
                    .get(&loc)
                    .map_err(|_| PasmError::InvalidIndirectAddress(loc))?;

                Ok(())
            }
            _ => Err(PasmError::InvalidMultiOp),
        },
        Op::Null => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}

/// Load value from memory using indexed addressing into register
///
/// # Syntax
///
/// 1. `LDM [loc]` - loads to `ACC`
/// 2. `LDM [reg],[loc]` - loads to `reg`
pub fn ldx(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        Op::Loc(mut loc) => {
            loc += ctx.ix;

            ctx.acc = ctx
                .mem
                .get(&loc)
                .map_err(|_| PasmError::InvalidIndexedAddress(loc))?;

            Ok(())
        }
        Op::MultiOp(ops) => match ops[..] {
            [ref reg, Op::Loc(mut loc)] if reg.is_register() => {
                loc += ctx.ix;

                *ctx.get_mut_register(reg) = ctx
                    .mem
                    .get(&loc)
                    .map_err(|_| PasmError::InvalidIndexedAddress(loc))?;

                Ok(())
            }
            _ => Err(PasmError::InvalidMultiOp),
        },
        Op::Null => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}

/// Load immediate value into `IX`
///
/// # Syntax
/// `LDR [lit]`
pub fn ldr(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        &Op::Literal(val) => ctx.ix = val,
        Op::Null => return Err(PasmError::NoOperand),
        _ => return Err(PasmError::InvalidOperand),
    }

    Ok(())
}

/// Move value from `ACC` to a register
/// OR
/// Move values between registers and memory locations
///
/// # Syntax
///
/// 1. `MOV [reg]` - move `ACC` value to `reg`
/// 2. `MOV [reg | loc],[reg | loc]` - move second value to first
pub fn mov(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        Op::MultiOp(ops) => match ops[..] {
            [ref dest, ref src] if dest.is_read_write() && src.is_usizeable() => {
                let src = src.get_val(ctx)?;
                ctx.modify(dest, |val| *val = src)?;
            }
            _ => return Err(PasmError::InvalidMultiOp),
        },
        reg if reg.is_register() => *ctx.get_mut_register(reg) = ctx.acc,
        Op::Null => return Err(PasmError::NoOperand),
        _ => return Err(PasmError::InvalidOperand),
    }

    Ok(())
}

/// Store `ACC` value in memory
///
/// # Syntax
/// `STO [loc]`
pub fn sto(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        Op::Loc(x) => {
            ctx.mem.write(x, ctx.acc)?;

            Ok(())
        }
        Op::Null => Err(PasmError::NoOperand),
        _ => Err(PasmError::InvalidOperand),
    }
}
