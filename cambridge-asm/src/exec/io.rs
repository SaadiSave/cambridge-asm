// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{exec::PasmError::*, inst};
use std::io::Read;

#[cfg(not(feature = "cambridge"))]
use crate::exec::{Context, PasmError, PasmResult};
#[cfg(not(feature = "cambridge"))]
use crate::inst::Op;

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

                write!(ctx.io.write, "{out}").expect("Unable to write to io");
            }
            src if src.is_usizeable() => {
                let src = src.get_val(ctx)?;

                if src > 255 {
                    return Err(InvalidUtf8Byte(src));
                }

                #[allow(clippy::cast_possible_truncation)]
                let out = src as u8 as char;

                write!(ctx.io.write, "{out}").expect("Unable to write to io");
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

                ctx.io
                    .read
                    .read_exact(&mut buf)
                    .expect("Unable to read from io");

                ctx.acc = buf[0] as usize;
            }
            dest if dest.is_read_write() => {
                let mut buf = [0; 1];

                ctx.io
                    .read
                    .read_exact(&mut buf)
                    .expect("Unable to read from io");

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
        let out = match op {
            Null => format!("{ctx:?}"),
            src if src.is_usizeable() => format!("{}", src.get_val(ctx)?),
            MultiOp(ops) if ops.iter().all(Op::is_usizeable) => ops
                .iter()
                .filter_map(|op| op.get_val(ctx).ok())
                .enumerate()
                .fold(String::new(), |acc, (idx, op)| {
                    if idx == ops.len() - 1 {
                        format!("{acc}{op}")
                    } else {
                        format!("{acc}{op}, ")
                    }
                }),
            MultiOp(_) => return Err(InvalidMultiOp),
            _ => return Err(InvalidOperand),
        };

        writeln!(ctx.io.write, "{out}").expect("Unable to write to io");
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
        const CR: u8 = 0xD;
        const LF: u8 = 0xA;

        fn read_line(reader: &mut impl std::io::Read, buf: &mut Vec<u8>) -> std::io::Result<()> {
            let mut prev = 0;
            let mut arr_buf = [0; 1];
            loop {
                reader.read_exact(&mut arr_buf)?;

                buf.extend_from_slice(&arr_buf);

                let current = arr_buf[0];

                if current == LF || [prev, current] == [CR, LF] {
                    break;
                }

                prev = arr_buf[0];
            }

            Ok(())
        }

        fn input(inp: &mut impl std::io::Read) -> usize {
            let mut buf = Vec::new();

            read_line(inp, &mut buf).expect("Unable to read stdin");

            let str = String::from_utf8_lossy(&buf);
            let str = str.trim();
            str.parse()
                .unwrap_or_else(|e| panic!("Unable to parse {str:?} because {e}"))
        }

        match op {
            Null => ctx.acc = input(&mut ctx.io.read),
            dest if dest.is_read_write() => {
                let input = input(&mut ctx.io.read);
                ctx.modify(dest, |d| *d = input)?;
            }
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
