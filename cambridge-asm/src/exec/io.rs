// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{exec::PasmError::*, inst};
use std::io::{BufRead, BufReader, Read};

fn failed_read<T>(err: impl std::error::Error) -> T {
    panic!("Unable to read from input because {err}")
}

fn failed_write<T>(err: impl std::error::Error) -> T {
    panic!("Unable to write to output because {err}")
}

inst!(
    /// No-op
    ///
    /// Start functions with this if you don't want to compromise readability
    ///
    /// # Syntax
    /// `NOP`
    pub nop {}
);

inst!(
    /// End a program
    /// Note that this is **NOT A NO-OP**. It will have effects on execution flow in code that uses functions
    pub end (ctx) {
        ctx.end = true;
    }
);

inst!(
    /// Output
    ///
    /// Convert an ASCII code to a character and print to STDOUT
    ///
    /// # Syntax
    /// 1. `OUT` - output `ACC`
    /// 2. `OUT [lit | reg | addr]`
    pub out (ctx, op) {
        match op {
            Null => {
                let x = ctx.acc;

                if x > 255 {
                    return Err(InvalidUtf8Byte(x));
                }

                #[allow(clippy::cast_possible_truncation)]
                let out = x as u8 as char;

                write!(ctx.io.write, "{out}").unwrap_or_else(failed_write);
            }
            src if src.is_usizeable() => {
                let src = src.get_val(ctx)?;

                if src > 255 {
                    return Err(InvalidUtf8Byte(src));
                }

                #[allow(clippy::cast_possible_truncation)]
                let out = src as u8 as char;

                write!(ctx.io.write, "{out}").unwrap_or_else(failed_write);
            }
            _ => return Err(InvalidOperand),
        }
    }
);

inst!(
    /// Input
    ///
    /// Read a single character from input, convert to ASCII code and
    /// store
    ///
    /// # Panics
    /// If error is encountered when reading input
    ///
    /// # Syntax
    /// 1. `INP` - read to `ACC`
    /// 2. `INP [reg | addr]`
    pub inp (ctx, op) {
        match op {
            Null => {
                let mut buf = [0; 1];

                ctx.io.read.read_exact(&mut buf).unwrap_or_else(failed_read);

                ctx.acc = buf[0] as usize;
            }
            dest if dest.is_read_write() => {
                let mut buf = [0; 1];

                ctx.io.read.read_exact(&mut buf).unwrap_or_else(failed_read);

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
    /// 2. `DBG [lit | reg | addr]` - print value
    /// 3. `DBG [lit | reg | addr], ...` - print value of all ops
    #[cfg(feature = "extended")]
    pub dbg (ctx, op) {
        let out = match op {
            Null => format!("{ctx:?}"),
            src if src.is_usizeable() => format!("{}", src.get_val(ctx)?),
            MultiOp(ops) if ops.iter().all(crate::inst::Op::is_usizeable) => ops
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

        writeln!(ctx.io.write, "{out}").unwrap_or_else(failed_write);
    }
);

// Raw input - directly input integers
inst!(
    /// Raw input
    /// Take integer input and store
    ///
    /// # Syntax
    /// 1. `RIN` - store to `ACC`
    /// 2. `RIN [reg | addr]`
    #[cfg(feature = "extended")]
    pub rin (ctx, op) {
        const LF: u8 = 0xA;

        fn input(inp: &mut BufReader<impl Read>) -> usize {
            let mut buf = Vec::with_capacity(32);
            inp.read_until(LF, &mut buf).unwrap_or_else(failed_read);

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

inst!(
    /// Call a function
    ///
    /// # Syntax
    /// `CALL [addr]`
    #[cfg(feature = "extended")]
    pub call (ctx, op) {
        match op {
            &Addr(addr) => {
                ctx.ret = ctx.mar + 1;
                ctx.override_flow_control();
                ctx.mar = addr;
            }
            _ => return Err(InvalidOperand),
        }
    }
);

inst!(
    /// Return to address in `Ar`
    ///
    /// # Syntax
    /// `RET`
    #[cfg(feature = "extended")]
    pub ret (ctx) {
        ctx.override_flow_control();
        ctx.mar = ctx.ret;
    }
);
