// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::exec::{Context, Executor, Inst, MemEntry, Memory, Op, OpFun, Source};
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;
use regex::Regex;
use std::{collections::BTreeMap, ops::Deref, path::Path};

#[derive(Parser)]
#[grammar = "pasm.pest"]
pub(crate) struct PasmParser;

pub type InstSet = fn(&str) -> Result<OpFun, String>;

pub(crate) struct Mem {
    pub addr: String,
    pub data: Option<String>,
}

impl Mem {
    pub fn new(addr: String, data: Option<String>) -> Self {
        Self { addr, data }
    }
}

struct Ir {
    pub addr: usize,
    pub inst: Inst,
}

impl Ir {
    pub fn new(addr: usize, inst: Inst) -> Self {
        Self { addr, inst }
    }
}

pub(crate) struct StrInst {
    pub addr: Option<String>,
    pub opcode: String,
    pub op: Option<String>,
}

impl StrInst {
    pub fn new(addr: Option<String>, opcode: String, op: Option<String>) -> Self {
        Self { addr, opcode, op }
    }
}

pub fn parse(prog: impl Deref<Target = str>, inst_set: InstSet) -> Executor {
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

    let src = Source::from(&*prog);
    debug!("This is your program code:\n{}", src);

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

    let exe = Executor::new(src, prog, Context::new(Memory::new(mem)));

    info!("Executor created");
    debug!("{}\n", exe);
    debug!("The initial context:\n{}\n", exe.ctx);

    exe
}

pub fn from_file(path: impl AsRef<Path>, inst_set: InstSet) -> Executor {
    let prog = std::fs::read_to_string(path).expect("Cannot read file");

    info!("File read complete.");

    parse(prog, inst_set)
}

/// Macro to generate an instruction set
#[macro_export]
macro_rules! inst_set {
    ($(#[$outer:meta])* $vis:vis $name:ident { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        $vis fn $name(op: &str) -> Result<$crate::exec::OpFun, String> {
            Ok(match op {
                $( $inst => $func,)+
                _ => return Err(format!("{op} is not an operation")),
            })
        }
    };
    ($(#[$outer:meta])* $vis:vis $name:ident $using:item { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        $vis fn $name(op: &str) -> Result<$crate::exec::OpFun, String> {
            $using
            Ok(match op {
                $( $inst => $func,)+
                _ => return Err(format!("{op} is not an operation")),
            })
        }
    };
    ($(#[$outer:meta])* $name:ident { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        fn $name(op: &str) -> Result<$crate::exec::OpFun, String> {
            Ok(match op {
                $( $inst => $func,)+
                _ => return Err(format!("{op} is not an operation")),
            })
        }
    };
    ($(#[$outer:meta])* $name:ident $using:item { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        fn $name(op: &str) -> Result<$crate::exec::OpFun, String> {
            $using
            Ok(match op {
                $( $inst => $func,)+
                _ => return Err(format!("{op} is not an operation")),
            })
        }
    };
}

/// Macro to extend any base instruction set
#[macro_export]
macro_rules! extend {
    ($(#[$outer:meta])* $vis:vis $name:ident extends $root:expr; { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        $vis fn $name(op: &str) -> Result<$crate::exec::OpFun, String> {
            Ok(match op {
                $( $inst => $func,)+
                _ => $root(op)?,
            })
        }
    };
    ($(#[$outer:meta])* $vis:vis $name:ident extends $root:expr, $using:item { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        $vis fn $name(op: &str) -> Result<$crate::exec::OpFun, String> {
            $using
            Ok(match op {
                $( $inst => $func,)+
                _ => $root(op)?,
            })
        }
    };
    ($(#[$outer:meta])* $name:ident extends $root:expr; { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        fn $name(op: &str) -> Result<$crate::exec::OpFun, String> {
            Ok(match op {
                $( $inst => $func,)+
                _ => $root(op)?,
            })
        }
    };
    ($(#[$outer:meta])* $name:ident extends $root:expr, $using:item { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        fn $name(op: &str) -> Result<$crate::exec::OpFun, String> {
            $using
            Ok(match op {
                $( $inst => $func,)+
                _ => $root(op)?,
            })
        }
    };
}

inst_set! {
    // Base instruction set
    pub get_fn use crate::exec::{mov, cmp, io, arith, bitman}; {
        "LDM" => mov::ldm,
        "LDD" => mov::ldd,
        "LDI" => mov::ldi,
        "LDX" => mov::ldx,
        "LDR" => mov::ldr,
        "MOV" => mov::mov,
        "STO" => mov::sto,

        "CMP" => cmp::cmp,
        "JPE" => cmp::jpe,
        "JPN" => cmp::jpn,
        "JMP" => cmp::jmp,
        "CMI" => cmp::cmi,

        "IN" => io::inp,
        "OUT" => io::out,
        "END" => io::end,

        "INC" => arith::inc,
        "DEC" => arith::dec,
        "ADD" => arith::add,
        "SUB" => arith::sub,

        "AND" => bitman::and,
        "OR" => bitman::or,
        "XOR" => bitman::xor,
        "LSL" => bitman::lsl,
        "LSR" => bitman::lsr,
    }
}

extend! {
    // Extended instruction set
    #[cfg(not(feature = "cambridge"))]
    pub get_fn_ext extends get_fn, use crate::exec::io; {
        "DBG" => io::dbg,
        "RIN" => io::rin,
        "CALL" => io::call,
        "RET" => io::ret,
        "NOP" => io::nop,
    }
}

fn get_inst(inst: Pair<Rule>) -> StrInst {
    let mut out = StrInst::new(None, "".into(), None);
    match inst.as_rule() {
        Rule::instruction => {
            let x = inst.into_inner();
            for i in x {
                match i.as_rule() {
                    Rule::address => out.addr = Some(i.as_str().into()),
                    Rule::label => {
                        out.addr = {
                            let x = i.as_str().to_string();
                            Some(x.replace(':', ""))
                        }
                    }
                    Rule::op => out.opcode = i.as_str().into(),
                    Rule::operand => out.op = Some(i.as_str().into()),
                    _ => panic!(
                        "{} is not an address, label, op, or operand token",
                        i.as_str()
                    ),
                }
            }
        }
        _ => panic!("Not an instruction"),
    }

    debug!(
        "{:>4}\t{:>4}\t{:<4}",
        out.addr.clone().unwrap_or_else(|| "".into()),
        out.opcode,
        out.op.clone().unwrap_or_else(|| "".into()),
    );
    out
}

pub(crate) fn get_insts(inst: Pairs<Rule>) -> Vec<StrInst> {
    let mut out = Vec::new();

    for pair in inst {
        for inner_pair in pair.into_inner() {
            out.push(get_inst(inner_pair));
        }
    }

    out
}

pub(crate) fn process_inst_links(insts: Vec<StrInst>) -> Vec<(usize, (String, Op))> {
    let mut links = Vec::new();

    let inst_list: Vec<_> = insts
        .into_iter()
        .map(|StrInst { addr, opcode, op }| {
            (
                addr,
                opcode,
                Op::from(op.map_or_else(|| "".into(), |s| s.trim().to_string())),
            )
        })
        .collect();

    for (i, (addr, _, _)) in inst_list.iter().enumerate() {
        for (j, (_, _, op)) in inst_list.iter().enumerate() {
            if addr.is_some() {
                match op {
                    Op::Loc(x) => {
                        if addr.as_ref().unwrap() == &x.to_string() {
                            links.push((i, j, None));
                        }
                    }
                    Op::Fail(x) => {
                        if addr.as_ref().unwrap() == x {
                            links.push((i, j, None));
                        }
                    }
                    Op::MultiOp(vec) => {
                        for (idx, op) in vec.iter().enumerate() {
                            match op {
                                Op::Loc(x) => {
                                    if addr.as_ref().unwrap() == &x.to_string() {
                                        links.push((i, j, Some(idx)));
                                    }
                                }
                                Op::Fail(x) => {
                                    if addr.as_ref().unwrap() == x {
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
    }

    debug!("Detected links within program:");
    debug!("{:?}\n", links);

    let mut ir = inst_list
        .into_iter()
        .enumerate()
        .map(|(i, j)| (i, (j.1, j.2)))
        .collect::<Vec<_>>();

    for i in links.clone() {
        match &ir[i.1].1 .1 {
            Op::MultiOp(ops) => {
                let mut ops = ops.clone();
                ops[i.2.unwrap()] = Op::Loc(i.0);
                ir[i.1].1 .1 = Op::MultiOp(ops);
            }
            Op::Loc(_) | Op::Fail(_) => ir[i.1].1 .1 = Op::Loc(i.0),
            _ => {}
        };
    }

    ir
}

fn process_insts(insts: Vec<StrInst>, inst_set: fn(&str) -> Result<OpFun, String>) -> Vec<Ir> {
    process_inst_links(insts)
        .into_iter()
        .map(|i| {
            Ir::new(
                i.0,
                Inst::new(
                    inst_set(&(i.1).0.to_uppercase()).unwrap_or_else(|s| panic!("{s}")),
                    (i.1).1,
                ),
            )
        })
        .collect()
}

fn get_mem(mem: Pair<Rule>) -> Mem {
    let mut out = Mem::new(String::new(), None);
    match mem.as_rule() {
        Rule::memory_entry => {
            let x = mem.into_inner();
            for i in x {
                match i.as_rule() {
                    Rule::address => out.addr = i.as_str().into(),
                    Rule::label => {
                        out.addr = {
                            let x = i.as_str().to_string();
                            x.replace(':', "")
                        }
                    }
                    Rule::data => out.data = Some(i.as_str().into()),
                    _ => panic!("{} is not an address, label or data", i.as_str()),
                }
            }
        }
        _ => panic!("Not an memory entry"),
    }

    debug!(
        "{:>4}\t{:<4}",
        out.addr,
        out.data.clone().unwrap_or_else(|| "".into())
    );
    out
}

pub(crate) fn get_mems(mem: Pairs<Rule>) -> Vec<Mem> {
    let mut out = Vec::new();

    for pair in mem {
        for inner_pair in pair.into_inner() {
            out.push(get_mem(inner_pair));
        }
    }

    out
}

fn process_mems(mems: &[Mem], prog: &mut Vec<Ir>) -> Vec<(usize, MemEntry)> {
    let mut links = Vec::new();

    for (i, Mem { addr, .. }) in mems.iter().enumerate() {
        for (
            j,
            Ir {
                inst: Inst { op, .. },
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
mod parse_tests {
    use crate::parse::parse;
    use std::time::Instant;

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
    fn test() {
        #[cfg(feature = "cambridge")]
        let parser = |prog: &str| parse(prog, crate::parse::get_fn);

        #[cfg(not(feature = "cambridge"))]
        let parser = |prog: &str| parse(prog, crate::parse::get_fn_ext);

        for (prog, acc) in PROGRAMS {
            let t = Instant::now();
            let mut exec = parser(prog);
            println!("{:?} elapsed", t.elapsed());
            exec.exec();
            assert_eq!(exec.ctx.acc, acc);
            println!("{:?} elapsed", t.elapsed());
        }
    }
}
