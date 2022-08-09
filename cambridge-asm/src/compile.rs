// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    exec::{Context, DebugInfo, ExecInst, Executor, Io, Memory},
    inst::{Inst, InstSet, Op},
    parse::{parse, Ir},
};
use std::{collections::BTreeMap, fmt::Display, ops::Deref, path::Path, str::FromStr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
struct CompiledInst {
    pub inst: String,
    pub op: Op,
}

impl CompiledInst {
    pub fn new(inst: String, op: Op) -> Self {
        Self { inst, op }
    }
}

type CompiledTree = BTreeMap<usize, CompiledInst>;

/// Represents a compiled program ready to be serialized into a file
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
pub struct CompiledProg {
    prog: CompiledTree,
    mem: Memory,
    debug_info: Option<DebugInfo>,
}

impl CompiledProg {
    fn new(prog: CompiledTree, mem: Memory, debug_info: Option<DebugInfo>) -> Self {
        Self {
            prog,
            mem,
            debug_info,
        }
    }

    /// Convert to an [`Executor`] so that program can be executed
    pub fn to_executor<T>(self, io: Io) -> Executor
    where
        T: InstSet,
        <T as FromStr>::Err: Display,
    {
        let prog = self
            .prog
            .into_iter()
            .map(|(addr, CompiledInst { inst, op })| {
                (
                    addr,
                    ExecInst::new(
                        (&inst)
                            .parse::<T>()
                            .unwrap_or_else(|s| panic!("{s}"))
                            .as_func_ptr(),
                        op,
                    ),
                )
            })
            .collect();

        Executor::new(
            "",
            prog,
            Context::with_io(self.mem, io),
            self.debug_info.unwrap_or_default(),
        )
    }
}

/// Parses source code into a [`CompiledProg`] ready for serialization
pub fn compile<T, P>(prog: P, debug: bool) -> CompiledProg
where
    T: InstSet,
    <T as FromStr>::Err: Display,
    P: Deref<Target = str>,
{
    let (insts, mem, _, debug_info) = parse(prog);

    let prog = insts
        .into_iter()
        .map(
            |Ir::<T> {
                 addr,
                 inst: Inst { inst, op },
             }| (addr, CompiledInst::new(inst.to_string(), op)),
        )
        .collect();

    let compiled = CompiledProg::new(prog, Memory::new(mem), debug.then_some(debug_info));

    info!("Program compiled");

    compiled
}

/// Parses source code into a [`CompiledProg`] directly from a file
pub fn from_file<T, P>(path: P, debug: bool) -> CompiledProg
where
    T: InstSet,
    <T as FromStr>::Err: Display,
    P: AsRef<Path>,
{
    let prog = std::fs::read_to_string(path).expect("Cannot read file");
    compile::<T, _>(prog, debug)
}

#[cfg(test)]
mod compile_tests {
    use crate::{
        compile::{compile, CompiledProg},
        make_io,
        parse::DefaultSet,
        TestStdout, PROGRAMS,
    };
    use std::time::Instant;

    #[test]
    pub fn test() {
        for (prog, res, out) in PROGRAMS {
            let mut t = Instant::now();

            let compiled = compile::<DefaultSet, _>(prog, false);
            let ser = serde_json::to_string(&compiled).unwrap();

            println!("Compilation time: {:?}", t.elapsed());

            t = Instant::now();
            let s = TestStdout::new(vec![]);

            let mut exe = serde_json::from_str::<CompiledProg>(&ser)
                .unwrap()
                .to_executor::<DefaultSet>(make_io!(std::io::stdin(), s.clone()));

            println!("JIT time: {:?}", t.elapsed());

            t = Instant::now();

            exe.exec::<DefaultSet>();

            println!("Execution time: {:?}", t.elapsed());

            assert_eq!(exe.ctx.acc, res);
            assert_eq!(s.to_vec(), out);
        }
    }
}
