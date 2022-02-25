// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{exec::PasmError::*, inst};
use std::io::Read;

#[cfg(not(feature = "cambridge"))]
use std::fmt::Display;

#[cfg(not(feature = "cambridge"))]
use crate::exec::{Context, Op, PasmError, PasmResult};

inst!(
    /// No-op
    /// Start functions with this if you don't want to compromise readability
    ///
    /// # Syntax
    /// `NOP`
    nop {}
);

inst!(
    /// End a program
    /// Note that this is **NOT A NO-OP**. It will have effects on execution flow in code that uses functions
    end | ctx
        | {
            ctx.end = true;
        }
);

inst!(
    /// Output
    /// Convert an ASCII code to a character and print to STDOUT
    ///
    /// # Syntax
    /// 1. `OUT` - output `ACC`
    /// 2. `OUT [lit | reg | loc]`
    out | ctx,
    op | {
        match op {
            Null => {
                let x = ctx.acc;

                if x > 255 {
                    return Err(InvalidUtf8Byte(x));
                }

                #[allow(clippy::cast_possible_truncation)]
                let out = x as u8 as char;

                print!("{out}");
            }
            src if src.is_usizeable() => {
                let src = src.get_val(ctx)?;

                if src > 255 {
                    return Err(InvalidUtf8Byte(src));
                }

                #[allow(clippy::cast_possible_truncation)]
                let out = src as u8 as char;

                print!("{out}");
            }
            _ => return Err(InvalidOperand),
        }
    }
);

inst!(
    /// Input
    /// Read a single character from STDIN, convert to ASCII code and store
    ///
    /// # Panics
    /// If error is encountered when reading STDIN
    ///
    /// # Syntax
    /// 1. `INP` - read to `ACC`
    /// 2. `INP [reg | loc]`
    inp | ctx,
    op | {
        match op {
            Null => {
                let mut buf = [0; 1];

                std::io::stdin()
                    .read_exact(&mut buf)
                    .expect("Unable to read STDIN");

                ctx.acc = buf[0] as usize;
            }
            dest if dest.is_read_write() => {
                let mut buf = [0; 1];

                std::io::stdin()
                    .read_exact(&mut buf)
                    .expect("Unable to read STDIN");

                ctx.modify(dest, |d| *d = buf[0] as usize)?;
            }
            _ => return Err(InvalidOperand),
        }
    }
);

// Custom instruction for debug logging
inst!(
    /// Print debug representation
    ///
    /// # Syntax
    /// 1. `DBG` - print entire execution context
    /// 2. `DBG [lit | reg | loc]` - print value
    /// 3. `DBG [lit | reg | loc], ...` - print value of all ops
    #[cfg(not(feature = "cambridge"))]
    dbg | ctx,
    op | {
        let out: Box<dyn Display> = match op {
            Null => Box::new(ctx),
            src if src.is_usizeable() => Box::new(src.get_val(ctx)?),
            MultiOp(ops) => Box::new({
                for src in ops {
                    if dbg(ctx, src).is_err() {
                        return Err(InvalidMultiOp);
                    }
                }
                ""
            }),
            _ => return Err(InvalidOperand),
        };

        println!("{out}");
    }
);

// Raw input - directly input integers
inst!(
    /// Raw input
    /// Take integer input and store
    ///
    /// # Syntax
    /// 1. `RIN` - store to `ACC`
    /// 2. `RIN [reg | loc]`
    #[cfg(not(feature = "cambridge"))]
    rin | ctx,
    op | {
        fn input() -> usize {
            let mut x = String::new();

            std::io::stdin()
                .read_line(&mut x)
                .expect("Unable to read stdin");

            x.trim()
                .parse()
                .unwrap_or_else(|_| panic!("'{x}' is not an integer"))
        }

        match op {
            Null => ctx.acc = input(),
            dest if dest.is_read_write() => ctx.modify(dest, |d| *d = input())?,
            _ => return Err(InvalidOperand),
        }
    }
);

/// Call a function
///
/// # Syntax
/// `CALL [loc]`
#[cfg(not(feature = "cambridge"))]
pub fn call(ctx: &mut Context, op: &Op) -> PasmResult {
    match op {
        &Op::Loc(loc) => {
            ctx.ret = ctx.mar + 1;
            ctx.override_flow_control();
            ctx.mar = loc;
            Ok(())
        }
        _ => Err(PasmError::InvalidOperand),
    }
}

/// Return to address in `Ar`
///
/// # Syntax
/// `RET`
#[cfg(not(feature = "cambridge"))]
pub fn ret(ctx: &mut Context, _: &Op) -> PasmResult {
    ctx.override_flow_control();
    ctx.mar = ctx.ret;
    Ok(())
}
