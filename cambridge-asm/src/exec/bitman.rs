// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, PasmError::*, PasmResult};
use crate::inst::Op::{self, *};

/// Bitwise AND
///
/// # Syntax
/// 1. `AND [lit | reg | addr]` - AND with `ACC`
/// 2. `AND [reg | addr],[lit | reg | addr]` - store second AND first to first
/// 3. `AND [reg | addr],[lit | reg | addr],[lit | reg | addr]` - store second AND third to first
pub fn and(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        MultiOp(ops) => match ops[..] {
            [ref dest, ref val] if dest.is_read_write() && val.is_usizeable() => {
                let val = val.get_val(ctx)?;
                ctx.modify(dest, |d| *d &= val)?;
            }
            [ref dest, ref a, ref b]
                if dest.is_read_write() && a.is_usizeable() && b.is_usizeable() =>
            {
                let val = a.get_val(ctx)? & b.get_val(ctx)?;
                ctx.modify(dest, |d| *d = val)?;
            }
            _ => return Err(InvalidMultiOp),
        },
        val if val.is_usizeable() => ctx.acc &= val.get_val(ctx)?,
        Null => return Err(NoOperand),
        _ => return Err(InvalidOperand),
    }

    Ok(())
}

/// Bitwise OR
///
/// # Syntax
/// 1. `OR [lit | reg | addr]` - OR with `ACC`
/// 2. `OR [reg | addr],[lit | reg | addr]` - store second OR first to first
/// 3. `OR [reg | addr],[lit | reg | addr],[lit | reg | addr]` - store second OR third to first
pub fn or(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        MultiOp(ops) => match ops[..] {
            [ref dest, ref val] if dest.is_read_write() && val.is_usizeable() => {
                let val = val.get_val(ctx)?;
                ctx.modify(dest, |d| *d |= val)?;
            }
            [ref dest, ref a, ref b]
                if dest.is_read_write() && a.is_usizeable() && b.is_usizeable() =>
            {
                let val = a.get_val(ctx)? | b.get_val(ctx)?;
                ctx.modify(dest, |d| *d = val)?;
            }
            _ => return Err(InvalidMultiOp),
        },
        val if val.is_usizeable() => ctx.acc |= val.get_val(ctx)?,
        Null => return Err(NoOperand),
        _ => return Err(InvalidOperand),
    }

    Ok(())
}

/// Bitwise XOR
///
/// # Syntax
/// 1. `XOR [lit | reg | addr]` - XOR with `ACC`
/// 2. `XOR [reg | addr],[lit | reg | addr]` - store second XOR first to first
/// 3. `XOR [reg | addr],[lit | reg | addr],[lit | reg | addr]` - store second XOR third to first
pub fn xor(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        MultiOp(ops) => match ops[..] {
            [ref dest, ref val] if dest.is_read_write() && val.is_usizeable() => {
                let val = val.get_val(ctx)?;
                ctx.modify(dest, |d| *d ^= val)?;
            }
            [ref dest, ref a, ref b]
                if dest.is_read_write() && a.is_usizeable() && b.is_usizeable() =>
            {
                let val = a.get_val(ctx)? ^ b.get_val(ctx)?;
                ctx.modify(dest, |d| *d = val)?;
            }
            _ => return Err(InvalidMultiOp),
        },
        val if val.is_usizeable() => ctx.acc ^= val.get_val(ctx)?,
        Null => return Err(NoOperand),
        _ => return Err(InvalidOperand),
    }

    Ok(())
}

/// Logical shift left
///
/// # Syntax
/// 1. `LSL [lit | reg | addr]` - LSL with `ACC`
/// 2. `LSL [reg | addr],[lit | reg | addr]` - store second LSL first to first
/// 3. `LSL [reg | addr],[lit | reg | addr],[lit | reg | addr]` - store second LSL third to first
pub fn lsl(ctx: &mut Context, op: &Op) -> PasmResult {
    #[allow(clippy::cast_possible_truncation)]
    fn checked_shl(dest: &mut usize, val: usize, mar: usize) {
        if let Some(res) = dest.checked_shl(val as u32) {
            *dest = res;
        } else {
            warn!("Shift left overflow detected at line {}", mar + 1);
            *dest <<= val;
        }
    }

    match op {
        MultiOp(ops) => {
            let line = ctx.mar;
            match ops[..] {
                [ref dest, ref val] if dest.is_read_write() && val.is_usizeable() => {
                    let val = val.get_val(ctx)?;
                    ctx.modify(dest, |d| checked_shl(d, val, line))
                }
                [ref dest, ref a, ref b]
                    if dest.is_read_write() && a.is_usizeable() && b.is_usizeable() =>
                {
                    let mut a = a.get_val(ctx)?;
                    checked_shl(&mut a, b.get_val(ctx)?, line);
                    ctx.modify(dest, |d| *d = a)
                }
                _ => Err(InvalidMultiOp),
            }
        }
        val if val.is_usizeable() => {
            let x = val.get_val(ctx)?;
            checked_shl(&mut ctx.acc, x, ctx.mar);
            Ok(())
        }
        Null => Err(NoOperand),
        _ => Err(InvalidOperand),
    }
}

/// Logical shift right
///
/// # Syntax
/// 1. `LSR [lit | reg | addr]` - LSR with `ACC`
/// 2. `LSR [reg | addr],[lit | reg | addr]` - store second LSR first to first
/// 3. `LSR [reg | addr],[lit | reg | addr],[lit | reg | addr]` - store second LSR third to first
pub fn lsr(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        MultiOp(ops) => match ops[..] {
            [ref dest, ref val] if dest.is_read_write() && val.is_usizeable() => {
                let val = val.get_val(ctx)?;
                ctx.modify(dest, |d| *d >>= val)
            }
            [ref dest, ref a, ref b]
                if dest.is_read_write() && a.is_usizeable() && b.is_usizeable() =>
            {
                let val = a.get_val(ctx)? >> b.get_val(ctx)?;
                ctx.modify(dest, |d| *d = val)
            }
            _ => Err(InvalidMultiOp),
        },
        val if val.is_usizeable() => {
            ctx.acc >>= val.get_val(ctx)?;
            Ok(())
        }
        Null => Err(NoOperand),
        _ => Err(InvalidOperand),
    }
}
