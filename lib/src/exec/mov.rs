// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, RtError::*, RtResult};
use crate::inst::Op::{self, *};

/// Load immediate values into a register
///
/// # Syntax
///
/// 1. `LDM [lit]` - loads to `ACC`
/// 2. `LDM [reg],[lit]` - loads to `reg`
pub fn ldm(ctx: &mut Context, op: &Op) -> RtResult {
    match op {
        MultiOp(ops) => match ops[..] {
            [ref op, Literal(val)] if op.is_register() => {
                *ctx.get_mut_register(op) = val;
                Ok(())
            }
            _ => Err(InvalidMultiOp),
        },
        &Literal(val) => {
            ctx.acc = val;
            Ok(())
        }
        Null => Err(NoOperand),
        _ => Err(InvalidOperand),
    }
}

/// Load values from memory into a register
///
/// # Syntax
///
/// 1. `LDD [addr]` - loads to `ACC`
/// 2. `LDD [reg],[addr]` - loads to `reg`
pub fn ldd(ctx: &mut Context, op: &Op) -> RtResult {
    match op {
        Addr(addr) => {
            ctx.acc = ctx.mem.get(addr).copied()?;
            Ok(())
        }
        MultiOp(ops) => match ops[..] {
            [ref reg, Addr(ref addr)] if reg.is_register() => {
                *ctx.get_mut_register(reg) = ctx.mem.get(addr).copied()?;
                Ok(())
            }
            _ => Err(InvalidMultiOp),
        },
        Null => Err(NoOperand),
        _ => Err(InvalidOperand),
    }
}

/// Load values from memory using indirect addressing into a register
///
/// # Syntax
///
/// 1. `LDM [addr]` - loads to `ACC`
/// 2. `LDM [reg],[addr]` - loads to `reg`
pub fn ldi(ctx: &mut Context, op: &Op) -> RtResult {
    match op {
        &Addr(addr) => {
            let addr2 = ctx.mem.get(&addr)?;

            ctx.acc = ctx
                .mem
                .get(addr2)
                .copied()
                .map_err(|_| InvalidIndirectAddr {
                    src: addr,
                    redirect: *addr2,
                })?;

            Ok(())
        }
        MultiOp(ops) => match ops[..] {
            [ref reg, Addr(addr)] if reg.is_register() => {
                let addr2 = ctx.mem.get(&addr)?;

                *ctx.get_mut_register(reg) =
                    ctx.mem
                        .get(addr2)
                        .copied()
                        .map_err(|_| InvalidIndirectAddr {
                            src: addr,
                            redirect: *addr2,
                        })?;

                Ok(())
            }
            _ => Err(InvalidMultiOp),
        },
        Null => Err(NoOperand),
        _ => Err(InvalidOperand),
    }
}

/// Load value from memory using indexed addressing into register
///
/// # Syntax
///
/// 1. `LDM [addr]` - loads to `ACC`
/// 2. `LDM [reg],[addr]` - loads to `reg`
pub fn ldx(ctx: &mut Context, op: &Op) -> RtResult {
    match op {
        &Addr(addr) => {
            ctx.acc = ctx
                .mem
                .get(&(addr + ctx.ix))
                .copied()
                .map_err(|_| InvalidIndexedAddr {
                    src: addr,
                    offset: ctx.ix,
                })?;

            Ok(())
        }
        MultiOp(ops) => match ops[..] {
            [ref reg, Addr(addr)] if reg.is_register() => {
                *ctx.get_mut_register(reg) =
                    ctx.mem
                        .get(&(addr + ctx.ix))
                        .copied()
                        .map_err(|_| InvalidIndexedAddr {
                            src: addr,
                            offset: ctx.ix,
                        })?;

                Ok(())
            }
            _ => Err(InvalidMultiOp),
        },
        Null => Err(NoOperand),
        _ => Err(InvalidOperand),
    }
}

/// Load immediate value into `IX`
///
/// # Syntax
/// `LDR [lit]`
pub fn ldr(ctx: &mut Context, op: &Op) -> RtResult {
    match op {
        &Literal(val) => ctx.ix = val,
        Null => return Err(NoOperand),
        _ => return Err(InvalidOperand),
    }

    Ok(())
}

/// Move value from `ACC` to a register
/// OR
/// Move values between registers and memory addresses
///
/// # Syntax
///
/// 1. `MOV [reg]` - move `ACC` value to `reg`
/// 2. `MOV [reg | addr],[reg | addr]` - move second value to first
pub fn mov(ctx: &mut Context, op: &Op) -> RtResult {
    match op {
        MultiOp(ops) => match ops[..] {
            [ref dest, ref src] if dest.is_read_write() && src.is_usizeable() => {
                let src = ctx.read(src)?;
                ctx.modify(dest, |val| *val = src)?;
            }
            _ => return Err(InvalidMultiOp),
        },
        reg if reg.is_register() => *ctx.get_mut_register(reg) = ctx.acc,
        Null => return Err(NoOperand),
        _ => return Err(InvalidOperand),
    }

    Ok(())
}

/// Store `ACC` value in memory
///
/// # Syntax
/// `STO [addr]`
pub fn sto(ctx: &mut Context, op: &Op) -> RtResult {
    match op {
        Addr(x) => {
            *ctx.mem.get_mut(x)? = ctx.acc;

            Ok(())
        }
        Null => Err(NoOperand),
        _ => Err(InvalidOperand),
    }
}
