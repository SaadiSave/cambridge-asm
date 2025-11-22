// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{exec::RtError::*, inst};
use std::io::{Read, Write};

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
    /// 3. `OUT [lit | reg | addr], ...` - output value in all ops as bytes
    pub out (ctx, op) {
        match op {
            Null => {
                ctx.io.write.write_all(&[ctx.acc.try_into().map_err(|_| InvalidUtf8Byte(ctx.acc))?])?;
            }
            src if src.is_usizeable() => {
                let src = ctx.read(src)?;

                ctx.io.write.write_all(&[src.try_into().map_err(|_| InvalidUtf8Byte(src))?])?;
            }
            MultiOp(ops) if ops.iter().all(inst::Op::is_usizeable) => for op in ops {
                out(ctx, op)?;
            }
            _ => return Err(InvalidOperand),
        }
    }
);

inst!(
    /// Print bytes from memory to stdout
    ///
    /// # Syntax
    /// `PRINT [addr], [n:lit]` - print `n` bytes from memory to stdout starting at address `addr`
    #[cfg(feature = "extended")]
    pub print (ctx, op) {
        match op {
            MultiOp(ops) => match &ops[..] {
                &[ref addr, Literal(n)] if addr.is_address() => {
                    let addr = ctx.as_address(addr)?;
                    let mut buf = Vec::with_capacity(n);

                    for address in addr..addr+n {
                        let byte = ctx.read(&Addr(address))?;

                        buf.push(byte.try_into().map_err(|_| InvalidUtf8Byte(byte))?);
                    }

                    ctx.io.write.write_all(&buf)?;
                }
                _ => return Err(InvalidMultiOp),
            }
            _ => return Err(InvalidOperand)
        }
    }
);

inst!(
    /// Input
    ///
    /// Read a single character from stdin, convert to ASCII code and
    /// store
    ///
    /// # Panics
    /// If error is encountered when reading stdin
    ///
    /// # Syntax
    /// 1. `IN` - read to `ACC`
    /// 2. `IN [reg | addr]`
    pub inp (ctx, op) {
        match op {
            Null => {
                let mut buf = [0; 1];

                ctx.io.read.read_exact(&mut buf)?;

                ctx.acc = buf[0] as usize;
            }
            dest if dest.is_read_write() => {
                let mut buf = [0; 1];

                ctx.io.read.read_exact(&mut buf)?;

                ctx.modify(dest, |d| *d = buf[0] as usize)?;
            }
            _ => return Err(InvalidOperand),
        }
    }
);

inst!(
    /// Read n bytes into memory
    ///
    /// # Syntax
    /// `READ [addr], [n:lit]` - read `n` bytes from stdin to memory starting at address `addr`
    #[cfg(feature = "extended")]
    pub read (ctx, op) {
        match op {
            MultiOp(ops) => match &ops[..] {
                &[ref start, Literal(n)] if start.is_address() => {
                    let start = ctx.as_address(start)?;

                    let mut buf = Vec::with_capacity(n);

                    ctx.io.read.read_exact(&mut buf)?;

                    for (start, byte) in (start..start+n).zip(buf) {
                        ctx.modify(&Addr(start), |d| *d = byte as usize)?;
                    }
                }
                _ => return Err(InvalidMultiOp)
            }
            _ => return Err(InvalidOperand)
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
            src if src.is_usizeable() => format!("{}", ctx.read(src)?),
            MultiOp(ops) if ops.iter().all(inst::Op::is_usizeable) => ops
                .iter()
                .filter_map(|op| ctx.read(op).ok())
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

        writeln!(ctx.io.write, "{out}")?;
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
        use std::io::BufRead;
        use super::RtResult;
        const LF: u8 = 0xA;

        fn input(inp: &mut impl BufRead) -> RtResult<usize> {
            let mut buf = Vec::with_capacity(32);
            inp.read_until(LF, &mut buf)?;

            let str = String::from_utf8_lossy(&buf);
            let str = str.trim();
            let res = str.parse()
                .map_err(|e| format!("Unable to parse {str:?} because {e}"))?;

            Ok(res)
        }

        match op {
            Null => ctx.acc = input(&mut ctx.io.read)?,
            dest if dest.is_read_write() => {
                let input = input(&mut ctx.io.read)?;
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
