// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::exec::{Cmd, Context, Executor, Func, Memory, Source};
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;
use std::ops::Deref;
use std::{collections::BTreeMap, path::Path};

#[derive(Parser)]
#[grammar = "pasm.pest"]
struct PasmParser;

type Inst = (Option<String>, String, Option<String>);
type FinInst = (usize, Cmd);
type Mem = (String, Option<String>);
type InstSet = fn(&str) -> Result<Func, String>;

#[must_use]
pub fn parse(prog: impl Deref<Target = str>, inst_set: InstSet) -> Executor {
    let line_ending = { prog.contains("\r\n").then(|| "\r\n").unwrap_or("\n") };

    let vec: Vec<_> = {
        let v: Vec<_> = prog
            .split(&format!("{}{}", line_ending, line_ending))
            .collect();

        if v.len() < 2 {
            panic!("Unable to parse. Your input may not contain one line between the program and the memory.");
        }

        v.iter()
            .map(|&s| {
                let mut x = s.to_owned();
                (!x.ends_with(line_ending)).then(|| x.push_str(line_ending));
                x
            })
            .collect()
    };

    let raw = Source::from(vec[0].as_str());
    debug!("This is your program:\n{:?}", &raw);

    let pairs = (
        PasmParser::parse(Rule::prog, &vec[0]).unwrap(),
        PasmParser::parse(Rule::memory, &vec[1]).unwrap(),
    );

    debug!("Instructions as detected:");
    debug!("Addr\tOpcode\tOp");
    debug!("-------\t-------\t-------");
    let insts = get_insts(pairs.0);

    debug!("Processing instructions into IR...");
    let mut insts = process_insts(&insts, inst_set);

    debug!("Memory as detected:");
    debug!("Addr\tData");
    debug!("-------\t-------");
    let mems = get_mems(pairs.1);

    debug!("Processing memory into IR...");
    let mems = process_mems(&mems, &mut insts);

    info!("Parsing complete. Creating executor...");

    let mut mem = BTreeMap::new();

    for i in mems {
        mem.insert(i.0, i.1);
    }

    let mut prog = BTreeMap::new();

    for i in insts {
        prog.insert(i.0, ((i.1).0, (i.1).1));
    }

    let exe = Executor::new(raw, Memory::new(prog), Context::new(Memory::new(mem), None));

    info!("Executor created.");
    debug!("Executor {:#?}\n", &exe);
    debug!("The initial context:\n{:#?}\n", &exe.ctx);

    exe
}

#[must_use]
pub fn from_file(path: &Path, inst_set: InstSet) -> Executor {
    let prog = std::fs::read_to_string(path).expect("Cannot read file");

    info!("File read complete.");

    parse(prog, inst_set)
}

/// Macro to generate an instruction set
#[macro_export]
macro_rules! inst_set {
    ($(#[$outer:meta])* $vis:vis $name:ident { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        $vis fn $name(op: &str) -> Result<$crate::exec::Func, String> {
            Ok(match op {
                $( $inst => $func,)+
                _ => return Err(format!("{} is not an operation", &op)),
            })
        }
    };
    ($(#[$outer:meta])* $vis:vis $name:ident $using:item { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        $vis fn $name(op: &str) -> Result<$crate::exec::Func, String> {
            $using
            Ok(match op {
                $( $inst => $func,)+
                _ => return Err(format!("{} is not an operation", &op)),
            })
        }
    };
    ($(#[$outer:meta])* $name:ident { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        fn $name(op: &str) -> Result<$crate::exec::Func, String> {
            Ok(match op {
                $( $inst => $func,)+
                _ => return Err(format!("{} is not an operation", &op)),
            })
        }
    };
    ($(#[$outer:meta])* $name:ident $using:item { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        fn $name(op: &str) -> Result<$crate::exec::Func, String> {
            $using
            Ok(match op {
                $( $inst => $func,)+
                _ => return Err(format!("{} is not an operation", &op)),
            })
        }
    };
}

/// Macro to extend any base instruction set
#[macro_export]
macro_rules! extension {
    ($(#[$outer:meta])* $vis:vis $name:ident extends $root:expr; { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        $vis fn $name(op: &str) -> Result<$crate::exec::Func, String> {
            Ok(match op {
                $( $inst => $func,)+
                _ => $root(op)?,
            })
        }
    };
    ($(#[$outer:meta])* $vis:vis $name:ident extends $root:expr, $using:item { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        $vis fn $name(op: &str) -> Result<$crate::exec::Func, String> {
            $using
            Ok(match op {
                $( $inst => $func,)+
                _ => $root(op)?,
            })
        }
    };
    ($(#[$outer:meta])* $name:ident extends $root:expr; { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        fn $name(op: &str) -> Result<$crate::exec::Func, String> {
            Ok(match op {
                $( $inst => $func,)+
                _ => $root(op)?,
            })
        }
    };
    ($(#[$outer:meta])* $name:ident extends $root:expr, $using:item { $( $inst:pat => $func:expr ),+ $(,)? }) => {
        $(#[$outer])*
        fn $name(op: &str) -> Result<$crate::exec::Func, String> {
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
        "CMPM" => cmp::cmpm,
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
        "ADDM" => arith::addm,
        "SUB" => arith::sub,
        "SUBM" => arith::subm,

        "AND" => bitman::and,
        "ANDM" => bitman::andm,
        "OR" => bitman::or,
        "ORM" => bitman::orm,
        "XOR" => bitman::xor,
        "XORM" => bitman::xorm,
        "LSL" => bitman::lsl,
        "LSR" => bitman::lsr,
    }
}

extension! {
    // Extended instruction set
    #[cfg(not(feature = "cambridge"))]
    pub get_fn_ext extends get_fn, use crate::exec::io; {
        "DBG" => io::dbg,
        "RIN" => io::rin,
    }
}

fn get_inst(inst: Pair<Rule>) -> Inst {
    let mut out: (Option<String>, String, Option<String>) = (None, "".into(), None);
    match inst.as_rule() {
        Rule::instruction => {
            let x = inst.into_inner();
            for i in x {
                match i.as_rule() {
                    Rule::address => out.0 = Some(i.as_str().into()),
                    Rule::label => {
                        out.0 = {
                            let x = i.as_str().to_string();
                            Some(x.replace(":", ""))
                        }
                    }
                    Rule::op => out.1 = i.as_str().into(),
                    Rule::operand => out.2 = Some(i.as_str().into()),
                    _ => panic!(
                        "{} is not an address, label, op, or operand token",
                        &i.as_str()
                    ),
                }
            }
        }
        _ => panic!("Not an instruction"),
    }

    if let Some(op) = out.2.clone() {
        if op.contains('#') {
            let oper = out.1.as_str();

            match oper {
                "ADD" => out.1 = "ADDM".into(),
                "SUB" => out.1 = "SUBM".into(),
                "AND" => out.1 = "ANDM".into(),
                "OR" => out.1 = "ORM".into(),
                "XOR" => out.1 = "XORM".into(),
                "CMP" => out.1 = "CMPM".into(),
                _ => {}
            }
        }
    }

    debug!(
        "{}\t{}\t{}",
        &out.0.clone().unwrap_or_else(|| String::from("None")),
        &out.1,
        &out.2.clone().unwrap_or_else(|| String::from("None"))
    );
    out
}

fn get_insts(inst: Pairs<Rule>) -> Vec<Inst> {
    let mut out = Vec::new();

    for pair in inst {
        for inner_pair in pair.into_inner() {
            out.push(get_inst(inner_pair));
        }
    }

    out
}

fn process_insts(insts: &[Inst], inst_set: fn(&str) -> Result<Func, String>) -> Vec<FinInst> {
    let mut links = Vec::new();

    for (i, (addr, _, _)) in insts.iter().enumerate() {
        for (j, (_, _, op)) in insts.iter().enumerate() {
            if addr.is_some() && op.is_some() && addr == op {
                links.push((i, j));
            }
        }
    }

    debug!("Detected links within program:\n{:?}\n", &links);

    let mut ir = Vec::new();

    for (i, j) in insts.iter().enumerate() {
        ir.push((i, (j.1.clone(), j.2.clone())));
    }

    for i in links {
        (ir[i.1].1).1 = Some(i.0.to_string());
    }

    let mut out = Vec::new();

    for i in ir {
        out.push((
            i.0,
            (
                inst_set(&(i.1).0.to_uppercase()).unwrap_or_else(|s| panic!("{}", s)),
                (i.1).1,
            ),
        ));
    }

    out
}

fn get_mem(mem: Pair<Rule>) -> Mem {
    let mut out = (String::new(), None);
    match mem.as_rule() {
        Rule::memoryentry => {
            let x = mem.into_inner();
            for i in x {
                match i.as_rule() {
                    Rule::address => out.0 = i.as_str().into(),
                    Rule::label => {
                        out.0 = {
                            let x = i.as_str().to_string();
                            x.replace(":", "")
                        }
                    }
                    Rule::data => out.1 = Some(i.as_str().into()),
                    _ => panic!("{} is not an address, label or data", &i.as_str()),
                }
            }
        }
        _ => panic!("Not an memory entry"),
    }

    debug!(
        "{}\t{}",
        &out.0,
        &out.1.clone().unwrap_or_else(|| String::from("None"))
    );
    out
}

fn get_mems(mem: Pairs<Rule>) -> Vec<Mem> {
    let mut out = Vec::new();

    for pair in mem {
        for inner_pair in pair.into_inner() {
            out.push(get_mem(inner_pair));
        }
    }

    out
}

fn process_mems(mems: &[Mem], prog: &mut Vec<FinInst>) -> Vec<(usize, usize)> {
    let mut links = Vec::new();

    for (i, (addr, _)) in mems.iter().enumerate() {
        for (j, (_, (_, op))) in prog.iter().enumerate() {
            if op.is_some() && addr.clone() == op.clone().unwrap() {
                links.push((i, j));
            }
        }
    }

    debug!("Detected links between program and memory:\n{:?}\n", &links);

    let mut out = Vec::new();

    for (i, j) in mems.iter().enumerate() {
        out.push((
            i,
            j.1.clone().unwrap_or_else(|| "0".into()).parse().unwrap(),
        ));
    }

    for i in links {
        (prog[i.1].1).1 = Some(i.0.to_string());
    }

    for i in prog.clone().iter().enumerate() {
        let mut finop = (i.1.clone().1).1;

        if let Some(mut op) = (i.1.clone().1).1 {
            if op.contains('#') {
                op.remove(0);

                finop = match op.chars().next().unwrap() {
                    'b' | 'B' => {
                        op.remove(0);
                        Some(usize::from_str_radix(&op, 2).unwrap().to_string())
                    }
                    'x' | 'X' => {
                        op.remove(0);
                        Some(usize::from_str_radix(&op, 16).unwrap().to_string())
                    }
                    '0'..='9' => Some(op.parse::<usize>().unwrap().to_string()),
                    _ => panic!("{} is an invalid operand", &op),
                }
            }
        }

        (prog[i.0].1).1 = finop;
    }

    out
}

#[cfg(test)]
#[cfg(not(feature = "cambridge"))]
#[test]
fn parse_test() {
    use std::path::PathBuf;

    let mut t = std::time::Instant::now();

    let mut exec = from_file(&PathBuf::from("examples/ex1.pasm"), get_fn_ext);
    exec.exec();
    println!("{:?}", t.elapsed());
    assert_eq!(exec.ctx.acc, 65);

    t = std::time::Instant::now();
    exec = from_file(&PathBuf::from("examples/ex2.pasm"), get_fn_ext);
    exec.exec();
    println!("{:?}", t.elapsed());
    assert_eq!(exec.ctx.acc, 15625);

    t = std::time::Instant::now();
    exec = from_file(&PathBuf::from("examples/ex3.pasm"), get_fn_ext);
    exec.exec();
    println!("{:?}", t.elapsed());
    assert_eq!(exec.ctx.acc, 10);
}
