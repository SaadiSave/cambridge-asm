// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    exec::{Context, DebugInfo, ExecInst, Executor, Io, Memory},
    inst::{InstSet, Op},
    parse::{parse, ErrorMap},
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
                        inst.parse::<T>()
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
pub fn compile<T>(prog: impl Deref<Target = str>, debug: bool) -> Result<CompiledProg, ErrorMap>
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    let (prog, mem, _, debug_info) = parse::<T>(prog)?;

    let prog = prog
        .into_iter()
        .map(|(addr, ExecInst { func, op })| {
            let str_inst = match T::from_func_ptr(func) {
                Ok(inst) => inst,
                Err(e) => panic!("{e}"),
            }
            .to_string();

            (addr, CompiledInst::new(str_inst, op))
        })
        .collect();

    let compiled = CompiledProg::new(prog, Memory::new(mem), debug.then_some(debug_info));

    info!("Program compiled");

    Ok(compiled)
}

/// Parses source code into a [`CompiledProg`] directly from a file
pub fn from_file<T>(path: impl AsRef<Path>, debug: bool) -> Result<CompiledProg, ErrorMap>
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    let prog = std::fs::read_to_string(path).expect("Cannot read file");
    compile::<T>(prog, debug)
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
    fn test() {
        for (prog, res, out) in PROGRAMS {
            let mut t = Instant::now();

            let compiled = compile::<DefaultSet>(prog, false).unwrap();
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
