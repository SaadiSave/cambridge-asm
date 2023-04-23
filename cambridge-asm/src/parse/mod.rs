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

inst_set! {
    /// The core instruction set
    ///
    /// Basic instructions only
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
    /// [`Core`], plus debugging, raw input, function call and return, and no-op instructions
    #[cfg(feature = "extended")]
    pub Extended extends Core use crate::exec::io; {
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

pub(crate) fn parse<T>(
    prog: impl Deref<Target = str>,
) -> (
    BTreeMap<usize, ExecInst>,
    BTreeMap<usize, usize>,
    Source,
    DebugInfo,
)
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    let (insts, mem, debug_info) = parser::Parser::<T>::new(&prog).parse().unwrap();
    let src = Source::from(prog);

    let mem = mem
        .into_iter()
        .map(|parser::MemIr { addr, data }| (addr, data))
        .collect();

    let prog = insts
        .into_iter()
        .map(|parser::InstIr::<T> { addr, inst }| (addr, inst.to_exec_inst()))
        .collect();

    (prog, mem, src, debug_info)
}

/// Parses a string into an [`Executor`]
///
/// This is the primary method to parse a pseudoassembly program
pub fn jit<T>(prog: impl Deref<Target = str>, io: Io) -> Executor
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    let (prog, mem, src, debug_info) = parse::<T>(prog);

    let exe = Executor::new(
        src,
        prog,
        Context::with_io(Memory::new(mem), io),
        debug_info,
    );

    info!("Executor created");
    debug!("{}\n", exe.display::<T>().unwrap_or_else(|s| panic!("{s}")));
    debug!("The initial context:\n{}\n", exe.ctx);

    exe
}

/// Parses a string into an [`Executor`] directly from a file
pub fn jit_from_file<T>(path: impl AsRef<Path>, io: Io) -> Executor
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
        TestStdout, PROGRAMS,
    };
    use std::time::Instant;

    #[test]
    fn test() {
        for (prog, acc, out) in PROGRAMS {
            let mut t = Instant::now();
            let s = TestStdout::new(vec![]);

            let mut exec = jit::<DefaultSet>(prog, make_io!(std::io::stdin(), s.clone()));

            println!("Parse time: {:?}", t.elapsed());

            t = Instant::now();

            exec.exec::<DefaultSet>();

            println!("Execution time: {:?}", t.elapsed());

            assert_eq!(exec.ctx.acc, acc);
            assert_eq!(s.to_vec(), out);
        }
    }

    #[test]
    #[should_panic]
    fn panics() {
        let mut exec = jit::<DefaultSet>(
            include_str!("../../examples/panics.pasm"),
            make_io!(std::io::stdin(), std::io::sink()),
        );
        exec.exec::<DefaultSet>();
    }
}