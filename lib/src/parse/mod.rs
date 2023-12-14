// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    exec::{Context, DebugInfo, ExecInst, Executor, Io, Memory, Source},
    extend,
    inst::InstSet,
    inst_set,
};
use std::{collections::BTreeMap, fmt::Display, ops::Deref, path::Path, str::FromStr};

mod lexer;
mod parser;

pub use lexer::{ErrorKind, ErrorMap, Span};

inst_set! {
    /// The core instruction set
    ///
    /// * Memory and register manipulation:
    /// `LDM`, `LDD`, `LDI`, `LDX`, `LDR`, `MOV`, `STO`
    ///
    /// * Comparison: `CMP`, `JPE`, `JPN`, `JMP`, `CMI`
    ///
    /// * Basic I/O: `IN`, `OUT`, `END`
    ///
    /// * Arithmetic: `INC`, `DEC`, `ADD`, `SUB`
    ///
    /// * Bit manipulation: `AND`, `OR`, `XOR`, `LSL`, `LSR`
    pub Core use crate::exec::{mov, cmp, io, arith, bitman}; {
        LDM => mov::ldm,
        LDD => mov::ldd,
        LDI => mov::ldi,
        LDX => mov::ldx,
        LDR => mov::ldr,
        MOV => mov::mov,
        STO => mov::sto,

        CMP => cmp::cmp,
        JPE => cmp::jpe,
        JPN => cmp::jpn,
        JMP => cmp::jmp,
        CMI => cmp::cmi,

        IN => io::inp,
        OUT => io::out,
        END => io::end,

        INC => arith::inc,
        DEC => arith::dec,
        ADD => arith::add,
        SUB => arith::sub,

        AND => bitman::and,
        OR => bitman::or,
        XOR => bitman::xor,
        LSL => bitman::lsl,
        LSR => bitman::lsr,
    }
}

extend! {
    /// The extended instruction set
    ///
    /// [`Core`], plus debugging (`DBG`), raw input (`RIN`), function `CALL` and return (`RET`), and no-op (`NOP`) instructions
    #[cfg(feature = "extended")]
    pub Extended extends Core use crate::exec::{io, arith::zero}; {
        ZERO => zero,
        DBG => io::dbg,
        RIN => io::rin,
        CALL => io::call,
        RET => io::ret,
        NOP => io::nop,
    }
}

// To make docs.rs ignore the feature cfgs
mod _default_set {
    #[cfg(not(feature = "extended"))]
    pub type DefaultSet = super::Core;

    #[cfg(feature = "extended")]
    pub type DefaultSet = super::Extended;
}

/// Depends on whether "extended" feature is enabled.
///
/// If enabled, it is `Extended`, otherwise `Core`.
pub type DefaultSet = _default_set::DefaultSet;

#[allow(clippy::type_complexity)]
pub(crate) fn parse<T>(
    prog: impl Deref<Target = str>,
) -> Result<
    (
        BTreeMap<usize, ExecInst>,
        BTreeMap<usize, usize>,
        Source,
        DebugInfo,
    ),
    ErrorMap,
>
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    let (insts, mem, debug_info) = parser::Parser::<T>::new(&prog).parse()?;
    let src = Source::from(prog);

    let mem = mem
        .into_iter()
        .map(|parser::MemIr { addr, data }| (addr, data))
        .collect();

    let prog = insts
        .into_iter()
        .map(|parser::InstIr::<T> { addr, inst }| (addr, inst.to_exec_inst()))
        .collect();

    Ok((prog, mem, src, debug_info))
}

/// Parse a string into an [`Executor`]
///
/// # Arguments
///
/// * `T`: instruction set
/// * `prog`: pseudo-assembly program
/// * `io`: I/O provider, use [`make_io`]
///
/// returns: `Result<Executor, ErrorMap>`
///
/// # Example
///
/// ```no_run
/// # use cambridge_asm::make_io;
/// # use cambridge_asm::parse::{ErrorMap, DefaultSet, jit};
///
/// # fn foo(s: String) -> Result<(), ErrorMap> {
/// let exec = jit::<DefaultSet>(s, make_io!())?;
/// # Ok(())
/// # }
/// ```
pub fn jit<T>(prog: impl Deref<Target = str>, io: Io) -> Result<Executor, ErrorMap>
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    let (prog, mem, src, debug_info) = parse::<T>(prog)?;

    let exe = Executor::new(
        src,
        prog,
        Context::with_io(Memory::new(mem), io),
        debug_info,
    );

    info!("Executor created");
    debug!("{}\n", exe.display::<T>().unwrap_or_else(|s| panic!("{s}")));
    debug!("The initial context:\n{}\n", exe.ctx);

    Ok(exe)
}

/// Parse a file into an [`Executor`]
///
/// # Arguments
///
/// * `T`: instruction set
/// * `path`: path to file containing pseudo-assembly program
/// * `io`: I/O provider, use [`make_io`]
///
/// returns: `Result<Executor, ErrorMap>`
///
/// # Example
///
/// ```no_run
/// # use cambridge_asm::make_io;
/// # use cambridge_asm::parse::{ErrorMap, DefaultSet, jit_from_file};
///
/// # fn foo(path: String) -> Result<(), ErrorMap> {
/// let exec = jit_from_file::<DefaultSet>(path, make_io!())?;
/// # Ok(())
/// # }
/// ```
pub fn jit_from_file<T>(path: impl AsRef<Path>, io: Io) -> Result<Executor, ErrorMap>
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    let prog = std::fs::read_to_string(path).expect("Cannot read file");

    info!("File read complete.");

    jit::<T>(prog, io)
}

#[cfg(test)]
mod parse_tests {
    use crate::{
        make_io,
        parse::{jit, DefaultSet},
        TestStdio, PROGRAMS,
    };
    use std::time::Instant;

    #[test]
    fn test() {
        for (prog, exp, inp, out) in PROGRAMS {
            let mut t = Instant::now();
            let s = TestStdio::new(vec![]);

            let mut exe =
                jit::<DefaultSet>(prog, make_io!(TestStdio::new(inp), s.clone())).unwrap();

            println!("Parse time: {:?}", t.elapsed());

            t = Instant::now();

            exe.exec::<DefaultSet>();

            println!("Execution time: {:?}", t.elapsed());

            assert_eq!(
                exe.ctx.acc, exp,
                "Expected '{}' in ACC, got '{}'",
                exp, exe.ctx.acc
            );
            assert_eq!(
                s.to_vec(),
                out,
                "Expected '{}' in output, got '{}'",
                String::from_utf8_lossy(out),
                s.try_to_string().unwrap()
            );
        }
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: {4..7: ParseIntError(ParseIntError { kind: InvalidDigit })}"
    )]
    fn panics() {
        let mut exec = jit::<DefaultSet>(
            include_str!("../../examples/panics.pasm"),
            make_io!(std::io::stdin(), std::io::sink()),
        )
        .unwrap();
        exec.exec::<DefaultSet>();
    }
}
