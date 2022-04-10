use crate::{
    exec::{Context, Executor, Inst, Io, MemEntry, Memory, Op},
    parse::{get_insts, get_mems, process_inst_links, InstSet, Mem, PasmParser, Rule, StrInst},
};
use pest::Parser;
use regex::Regex;
use std::{collections::BTreeMap, ops::Deref, path::Path};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
pub struct CompiledInst {
    pub opfun: String,
    pub op: Op,
}

impl CompiledInst {
    pub fn new(opfun: String, op: Op) -> Self {
        Self { opfun, op }
    }
}

pub type CompiledTree = BTreeMap<usize, CompiledInst>;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
pub struct CompiledProg {
    pub prog: CompiledTree,
    pub mem: Memory,
}

impl CompiledProg {
    pub fn new(prog: CompiledTree, mem: Memory) -> Self {
        Self { prog, mem }
    }

    pub fn to_executor(self, inst_set: InstSet, io: Io) -> Executor {
        let prog = self
            .prog
            .into_iter()
            .map(|(addr, CompiledInst { opfun, op })| {
                (
                    addr,
                    Inst::new(inst_set(&opfun).unwrap_or_else(|s| panic!("{s}")), op),
                )
            })
            .collect();

        Executor::new("", prog, Context::with_io(self.mem, io))
    }
}

struct Ir {
    pub addr: usize,
    pub inst: CompiledInst,
}

impl Ir {
    pub fn new(addr: usize, inst: CompiledInst) -> Self {
        Self { addr, inst }
    }
}

pub fn compile(prog: impl Deref<Target = str>, inst_set: InstSet) -> CompiledProg {
    let mut line_ending = if prog.contains("\r\n") {
        // Windows
        r"\r\n"
    } else if prog.contains('\r') {
        // For old Macs
        r"\r"
    } else {
        // UNIX
        r"\n"
    };

    let separator = Regex::new(&format!("{line_ending} *{line_ending} *")).unwrap();

    line_ending = match line_ending {
        r"\r\n" => "\r\n",
        r"\n" => "\n",
        r"\r" => "\r",
        _ => unreachable!(),
    };

    let mut vec: Vec<_> = {
        let v: Vec<_> = separator.split(&prog).collect();

        assert!((v.len() >= 2), "Unable to parse. Your input may not contain blank line(s) between the program and the memory.");

        v.iter()
            .map(|&s| {
                let mut x = s.to_string();
                (!x.ends_with(line_ending)).then(|| x.push_str(line_ending));
                x
            })
            .collect()
    };

    let mem = vec.pop().unwrap();
    let prog = vec.join("");

    let pairs = (
        PasmParser::parse(Rule::prog, &prog).unwrap(),
        PasmParser::parse(Rule::memory, &mem).unwrap(),
    );

    debug!("Instructions as detected:");
    debug!("Addr\tOpcode\tOp");
    debug!("{:-<7}\t{:-<7}\t{:-<7}", "-", "-", "-");
    let insts = get_insts(pairs.0);

    debug!("Processing instructions into IR...");
    let mut insts = process_insts(insts, inst_set);

    debug!("Memory as detected:");
    debug!("Addr\tData");
    debug!("{:-<7}\t{:-<7}", "-", "-");
    let mems = get_mems(pairs.1);

    debug!("Processing memory into IR...");
    let mems = process_mems(&mems, &mut insts);

    info!("Parsing complete. Creating executor...");

    let mem = BTreeMap::from_iter(mems);

    let prog = insts
        .into_iter()
        .map(|Ir { addr, inst }| (addr, inst))
        .collect();

    let exe = CompiledProg::new(prog, Memory::new(mem));

    info!("Program compiled");

    exe
}

pub fn from_file(path: impl AsRef<Path>, inst_set: InstSet) -> CompiledProg {
    let prog = std::fs::read_to_string(path).expect("Cannot read file");
    compile(prog, inst_set)
}

fn process_insts(insts: Vec<StrInst>, inst_set: InstSet) -> Vec<Ir> {
    process_inst_links(insts)
        .into_iter()
        .map(|(adrr, (opfun, op))| {
            Ir::new(
                adrr,
                CompiledInst::new(
                    {
                        assert!(
                            inst_set(&opfun.to_uppercase()).is_ok(),
                            "{opfun} is not a valid op"
                        );
                        opfun.to_uppercase()
                    },
                    op,
                ),
            )
        })
        .collect()
}

fn process_mems(mems: &[Mem], prog: &mut Vec<Ir>) -> Vec<(usize, MemEntry)> {
    let mut links = Vec::new();

    for (i, Mem { addr, .. }) in mems.iter().enumerate() {
        for (
            j,
            Ir {
                inst: CompiledInst { op, .. },
                ..
            },
        ) in prog.iter().enumerate()
        {
            match op {
                Op::Loc(x) => {
                    if addr == &x.to_string() {
                        links.push((i, j, None));
                    }
                }
                Op::Fail(x) => {
                    if addr == x {
                        links.push((i, j, None));
                    }
                }
                Op::MultiOp(vec) => {
                    for (idx, op) in vec.iter().enumerate() {
                        match op {
                            Op::Loc(x) => {
                                if addr == &x.to_string() {
                                    links.push((i, j, Some(idx)));
                                }
                            }
                            Op::Fail(x) => {
                                if addr == x {
                                    links.push((i, j, Some(idx)));
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    debug!("Detected links between program and memory:");
    debug!("{:?}\n", links);

    // linking
    for i in links.clone() {
        match &prog[i.1].inst.op {
            Op::MultiOp(ops) => {
                let mut ops = ops.clone();
                ops[i.2.unwrap()] = Op::Loc(i.0);
                prog[i.1].inst.op = Op::MultiOp(ops);
            }
            Op::Loc(_) | Op::Fail(_) => prog[i.1].inst.op = Op::Loc(i.0),
            _ => {}
        };
    }

    let mut memlinks = Vec::new();

    for (i, Mem { addr, .. }) in mems.iter().enumerate() {
        for (j, Mem { data, .. }) in mems.iter().enumerate() {
            if let Some(o) = data {
                if addr == o {
                    memlinks.push((i, j));
                }
            }
        }
    }

    debug!("Detected links within memory:");
    debug!("{:?}\n", memlinks);

    let mut ir = mems
        .iter()
        .enumerate()
        .map(|(i, j)| {
            (
                i,
                MemEntry::new(
                    j.data
                        .clone()
                        .unwrap_or_else(|| "0".to_string())
                        .parse()
                        .unwrap(),
                ),
            )
        })
        .collect::<Vec<_>>();

    for i in memlinks {
        ir[i.1].1.address = Some(i.0);
    }

    ir
}

#[cfg(test)]
mod compile_tests {
    use crate::{
        compile::{compile, CompiledProg},
        make_io,
    };

    #[cfg(feature = "cambridge")]
    const PROGRAMS: [(&str, usize); 1] = [(include_str!("../examples/hello.pasm"), 207)];

    #[cfg(not(feature = "cambridge"))]
    const PROGRAMS: [(&str, usize); 4] = [
        (include_str!("../examples/division.pasm"), 65),
        (include_str!("../examples/multiplication.pasm"), 15625),
        (include_str!("../examples/hello.pasm"), 207),
        (include_str!("../examples/functions.pasm"), 65),
    ];

    #[test]
    pub fn test() {
        #[cfg(feature = "cambridge")]
        let inst_set = crate::parse::get_fn;

        #[cfg(not(feature = "cambridge"))]
        let inst_set = crate::parse::get_fn_ext;

        for (prog, res) in PROGRAMS {
            let compiled = compile(prog, inst_set);
            let ser = serde_json::to_string(&compiled).unwrap();

            println!("{ser}");

            let t = std::time::Instant::now();
            let mut exe = serde_json::from_str::<CompiledProg>(&ser)
                .unwrap()
                .to_executor(inst_set, make_io!(std::io::stdin(), std::io::sink()));
            println!("{:?} elapsed", t.elapsed());

            exe.exec();
            assert_eq!(exe.ctx.acc, res);
            println!("{:?} elapsed", t.elapsed());
        }
    }
}
