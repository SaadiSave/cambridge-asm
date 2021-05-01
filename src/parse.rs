// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use pest::{Parser, iterators::{Pair, Pairs}};
use crate::exec::{Context, Cmd, Executor, Func, self, Memory};
use std::{collections::BTreeMap, path::Path};

#[derive(Parser)]
#[grammar = "pasm.pest"]
struct PasmParser;

type Inst = (Option<String>, String, Option<String>);
type FinInst = (usize, Cmd);
type Mem = (String, Option<String>);

pub fn parse(path: &Path) -> Executor {
    let x = std::fs::read_to_string(path).expect("File cannot be read");

    let vec: Vec<_> = x.split("\n\n").collect();

    let pairs = (PasmParser::parse(Rule::prog, vec[0]).unwrap(), PasmParser::parse(Rule::memory, vec[1]).unwrap());

    let instructions = get_insts(pairs.0);

    let mut exec = process_insts(&instructions);
    
    let mems = get_mems(pairs.1);

    let mems = process_mems(&mems, &mut exec);

    let mut mem = BTreeMap::new();

    for i in mems {
        mem.insert(i.0, i.1);
    }

    let mut prog = BTreeMap::new();

    for i in exec {
        prog.insert(i.0, ((i.1).0, (i.1).1));
    }

    Executor {
        prog: Memory(prog),
        ctx: Context { cmpr: false, mar: 0, acc: 0, ix: 0, mem: Memory(mem)},
    }
}

fn get_fn(op: &str) -> Func {
    use exec::{mov, cmp, io, arith, bitman};
    match op {
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
        "DBG" => io::dbg,
        "RIN" => io::rin,
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

        _ => panic!("{} is not an operation", &op),
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
                    Rule::label => out.0 = {
                        let x = i.as_str().to_string();
                        Some(x.replace(":", ""))
                    },
                    Rule::op => out.1 = i.as_str().into(),
                    Rule::operand => out.2 = Some(i.as_str().into()),
                    _ => panic!("{} is not an address, label, op, or operand token", &i.as_str()),
                }
            }
        },
        _ => panic!("Not an instruction")
    }

    if let Some(mut op) = out.2.clone() {
        if op.contains('#') {
            op.remove(0);

            match op.chars().next().unwrap() {
                'b' | 'B' => out.2 = {
                    op.remove(0);
                    Some(usize::from_str_radix(&op, 2).unwrap().to_string())
                },
                'x' | 'X' => out.2 = {
                    op.remove(0);
                    Some(usize::from_str_radix(&op, 16).unwrap().to_string())
                },
                '0'..='9' => out.2 = Some(op.parse::<usize>().unwrap().to_string()),
                _ => panic!("{} is an invalid operand", &op),
            }

            let oper = out.1.as_str();

            match oper {
                "ADD" => out.1 = "ADDM".into(),
                "SUB" => out.1 = "SUBM".into(),
                "AND" => out.1 = "ANDM".into(),
                "OR" => out.1 = "ORM".into(),
                "XOR" => out.1 = "XORM".into(),
                "CMP" => out.1 = "CMPM".into(),
                _ => {},
            }
        }
    }

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

fn process_insts(insts: &[Inst]) -> Vec<FinInst> {
    let mut links = Vec::new();

    for (i, (addr, _, _)) in insts.iter().enumerate() {
        for (j, (_, _, op)) in insts.iter().enumerate() {
            if addr.is_some() && op.is_some() && addr == op {
                links.push((i, j));
            }
        }
    }

    let mut int = Vec::new();

    for (i, j) in insts.iter().enumerate() {
        int.push((i, (j.1.clone(), j.2.clone())));
    }

    for i in links {
        (int[i.1].1).1 = Some(i.0.to_string());
    }

    let mut out = Vec::new();

    for i in int {
        out.push((i.0, (get_fn(&(i.1).0), (i.1).1)))
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
                    Rule::label => out.0 = {
                        let x = i.as_str().to_string();
                        x.replace(":", "")
                    },
                    Rule::data => out.1 = Some(i.as_str().into()),
                    _ => panic!("{} is not an address, label or data", &i.as_str()),
                }
            }
        },
        _ => panic!("Not an memory entry")
    }

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

    let mut int = Vec::new();

    for (i, j) in mems.iter().enumerate() {
        int.push((i, j.1.clone().unwrap_or_else(|| "0".into()).parse().unwrap()));
    }

    for i in links {
        (prog[i.1].1).1 = Some(i.0.to_string());
    }

    int
}

#[test]
fn parse_test() {
    let t = std::time::Instant::now();

    let mut exec = parse(&std::path::PathBuf::from("examples/ex1.pasm"));
    println!("\n{:?}", &t.elapsed());
    exec.exec();
    println!("\n{:?}", &t.elapsed());

    let t = std::time::Instant::now();
    
    let mut exec = parse(&std::path::PathBuf::from("examples/ex2.pasm"));
    println!("\n{:?}", &t.elapsed());
    exec.exec();
    println!("\n{:?}", &t.elapsed());
}
