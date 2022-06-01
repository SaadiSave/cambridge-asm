// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, PasmError::*, PasmResult};
use crate::inst::Op::{self, *};

#[inline]
fn checked_add(dest: &mut usize, val: usize, mar: usize) {
    if let Some(res) = dest.checked_add(val) {
        *dest = res;
    } else {
        warn!("Addition overflow detected at line {}", mar + 1);
        *dest += val;
    }
}

/// Add values
///
/// # Syntax
/// 1. `ADD [lit | reg | addr]` - add to `ACC`
/// 2. `ADD [reg | addr],[lit | reg | addr]` - add second value to first
/// 3. `ADD [reg | addr],[lit | reg | addr],[lit | reg | addr]` - add second and third value, store to first
pub fn add(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        MultiOp(ops) => match ops[..] {
            [ref dest, ref val] if dest.is_read_write() && val.is_usizeable() => {
                let line = ctx.mar;
                let val = val.get_val(ctx)?;
                ctx.modify(dest, |d| checked_add(d, val, line))?;
            }
            [ref dest, ref a, ref b]
                if dest.is_read_write() && a.is_usizeable() && b.is_usizeable() =>
            {
                let mut a = a.get_val(ctx)?;
                checked_add(&mut a, b.get_val(ctx)?, ctx.mar);
                ctx.modify(dest, |d| *d = a)?;
            }
            _ => return Err(InvalidMultiOp),
        },
        Null => return Err(NoOperand),
        val if val.is_usizeable() => {
            let val = val.get_val(ctx)?;
            checked_add(&mut ctx.acc, val, ctx.mar);
        }
        _ => return Err(InvalidOperand),
    }

    Ok(())
}

#[inline]
fn checked_sub(dest: &mut usize, val: usize, mar: usize) {
    if let Some(res) = dest.checked_sub(val) {
        *dest = res;
    } else {
        warn!("Subtraction overflow detected at line {}", mar + 1);
        *dest -= val;
    }
}

/// Subtract values
///
/// # Syntax
/// 1. `ADD [lit | reg | addr]` - subtract from `ACC`
/// 2. `ADD [reg | addr],[lit | reg | addr]` - subtract second value from first
/// 3. `ADD [reg | addr],[lit | reg | addr],[lit | reg | addr]` - subtract third from second value, store to first
pub fn sub(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        MultiOp(ops) => match ops[..] {
            [ref dest, ref val] if dest.is_read_write() && val.is_usizeable() => {
                let line = ctx.mar;
                let val = val.get_val(ctx)?;
                ctx.modify(dest, |d| checked_sub(d, val, line))?;
            }
            [ref dest, ref a, ref b]
                if dest.is_read_write() && a.is_usizeable() && b.is_usizeable() =>
            {
                let mut a = a.get_val(ctx)?;
                checked_sub(&mut a, b.get_val(ctx)?, ctx.mar);
                ctx.modify(dest, |d| *d = a)?;
            }
            _ => return Err(InvalidMultiOp),
        },
        val if val.is_usizeable() => {
            let val = val.get_val(ctx)?;
            checked_sub(&mut ctx.acc, val, ctx.mar);
        }
        Null => return Err(NoOperand),
        _ => return Err(InvalidOperand),
    }

    Ok(())
}

/// Increment register or memory address
///
/// # Syntax
/// `INC [reg | addr]`
pub fn inc(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        dest if dest.is_read_write() => {
            let line = ctx.mar;
            ctx.modify(dest, |d| checked_add(d, 1, line))?;
        }
        Null => return Err(NoOperand),
        _ => return Err(InvalidOperand),
    }

    Ok(())
}

/// Decrement register or memory address
///
/// # Syntax
/// `DEC [reg | addr]`
pub fn dec(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        dest if dest.is_read_write() => {
            let line = ctx.mar;
            ctx.modify(dest, |d| checked_sub(d, 1, line))?;
        }
        Null => return Err(NoOperand),
        _ => return Err(InvalidOperand),
    }

    Ok(())
}
