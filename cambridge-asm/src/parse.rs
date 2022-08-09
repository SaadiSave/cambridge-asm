// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    exec::{Context, DebugInfo, Executor, Io, Memory, Source},
    extend,
    inst::{Inst, InstSet, Op},
    inst_set,
};
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use regex::Regex;
use std::{collections::BTreeMap, fmt::Display, ops::Deref, path::Path, str::FromStr};

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

// pest derive macro makes `Rule` enum public, so conceal in module
mod _private {
    #[derive(pest_derive::Parser)]
    #[grammar = "pasm.pest"]
    pub struct PasmParser;
}

use _private::{PasmParser, Rule};

struct Mem {
    pub addr: String,
    pub data: Option<String>,
}

impl Mem {
    pub fn new(addr: String, data: Option<String>) -> Self {
        Self { addr, data }
    }
}

pub(crate) struct Ir<T>
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    pub addr: usize,
    pub inst: Inst<T>,
}

impl<T> Ir<T>
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    pub fn new(addr: usize, inst: Inst<T>) -> Self {
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

pub(crate) fn parse<T, P>(prog: P) -> (Vec<Ir<T>>, BTreeMap<usize, usize>, Source, DebugInfo)
where
    T: InstSet,
    <T as FromStr>::Err: Display,
    P: Deref<Target = str>,
{
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

    // unwrap is ok, because regex is valid
    let separator = Regex::new(&format!("{line_ending} *{line_ending} *")).unwrap();

    line_ending = match line_ending {
        r"\r\n" => "\r\n",
        r"\n" => "\n",
        r"\r" => "\r",
        _ => unreachable!(), // ok, because line_ending cannot be anything else
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

    // unwrap is ok, because vec.len() >= 2
    let mem = vec.pop().unwrap();
    let prog = vec.join("");

    let src = Source::from(&*prog);
    debug!("This is your program code:\n{}", src);

    let pairs = (
        PasmParser::parse(Rule::prog, &prog).unwrap(),
        PasmParser::parse(Rule::memory, &mem).unwrap(),
    );

    let mut debug_info = DebugInfo::default();

    debug!("Instructions as detected:");
    debug!("Addr\tOpcode\tOp");
    debug!("{:-<7}\t{:-<7}\t{:-<7}", "-", "-", "-");
    let insts = get_insts(pairs.0);

    debug!("Processing instructions into IR...");
    let mut insts = process_insts::<T>(insts, &mut debug_info);

    debug!("Memory as detected:");
    debug!("Addr\tData");
    debug!("{:-<7}\t{:-<7}", "-", "-");
    let mems = get_mems(pairs.1);

    debug!("Processing memory into IR...");
    let mems = process_mems::<T>(mems, &mut insts, &mut debug_info);

    (insts, BTreeMap::from_iter(mems), src, debug_info)
}

/// Parses a string into an [`Executor`]
///
/// This is the primary method to parse a pseudoassembly program
pub fn jit<T, P>(prog: P, io: Io) -> Executor
where
    T: InstSet,
    <T as FromStr>::Err: Display,
    P: Deref<Target = str>,
{
    let (insts, mem, src, debug_info) = parse(prog);

    let prog = insts
        .into_iter()
        .map(|Ir::<T> { addr, inst }| (addr, inst.to_exec_inst()))
        .collect();

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
pub fn jit_from_file<T, P>(path: P, io: Io) -> Executor
where
    T: InstSet,
    <T as FromStr>::Err: Display,
    P: AsRef<Path>,
{
    let prog = std::fs::read_to_string(path).expect("Cannot read file");

    info!("File read complete.");

    jit::<T, String>(prog, io)
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

fn get_insts(inst: Pairs<Rule>) -> Vec<StrInst> {
    let mut out = Vec::new();

    for pair in inst {
        for inner_pair in pair.into_inner() {
            out.push(get_inst(inner_pair));
        }
    }

    out
}

fn process_inst_links(
    insts: Vec<StrInst>,
    debug_info: &mut DebugInfo,
) -> Vec<(usize, (String, Op))> {
    let mut links = Vec::new();

    let inst_list = insts
        .into_iter()
        .map(|StrInst { addr, opcode, op }| {
            (
                addr,
                opcode,
                Op::from(op.map_or_else(|| "".into(), |s| s.trim().to_string())),
            )
        })
        .collect::<Vec<_>>();

    debug_info.prog.extend(
        inst_list
            .iter()
            .enumerate()
            .filter_map(|(addr, (label, _, _))| label.as_ref().map(|label| (addr, label.clone()))),
    );

    // find links
    for (i, (addr, _, _)) in inst_list.iter().enumerate() {
        for (j, (_, _, op)) in inst_list.iter().enumerate() {
            if let Some(addr) = addr {
                match op {
                    Op::Addr(x) if addr == &x.to_string() => links.push((i, j, None)),
                    Op::Fail(x) if addr == x => links.push((i, j, None)),
                    Op::MultiOp(vec) => {
                        for (idx, op) in vec.iter().enumerate() {
                            match op {
                                #[rustfmt::skip]
                                Op::Addr(x) if addr == &x.to_string() => links.push((i, j, Some(idx))),
                                Op::Fail(x) if addr == x => links.push((i, j, Some(idx))),
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
        .map(|(idx, inst)| (idx, (inst.1, inst.2, inst.0)))
        .collect::<Vec<_>>();

    // linking
    for (to, from, multiop_idx) in links {
        match &ir[from].1 .1 {
            Op::MultiOp(ops) => {
                let mut ops = ops.clone();
                // unwrap ok because mutiop_idx will always exist if operand is multiop
                ops[multiop_idx.unwrap()] = Op::Addr(to);
                ir[from].1 .1 = Op::MultiOp(ops);
            }
            Op::Addr(_) | Op::Fail(_) => ir[from].1 .1 = Op::Addr(to),
            _ => {}
        };
    }

    ir.into_iter()
        .map(|(addr, (inst, op, _))| (addr, (inst, op)))
        .collect()
}

fn process_insts<T>(insts: Vec<StrInst>, debug_info: &mut DebugInfo) -> Vec<Ir<T>>
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    process_inst_links(insts, debug_info)
        .into_iter()
        .map(|(idx, (inst, op))| {
            Ir::new(
                idx,
                Inst::new(
                    (inst.to_uppercase())
                        .parse()
                        .unwrap_or_else(|s| panic!("{s}")),
                    op,
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

fn get_mems(mem: Pairs<Rule>) -> Vec<Mem> {
    let mut out = Vec::new();

    for pair in mem {
        for inner_pair in pair.into_inner() {
            out.push(get_mem(inner_pair));
        }
    }

    out
}

fn process_mems<T>(
    mems: Vec<Mem>,
    prog: &mut [Ir<T>],
    debug_info: &mut DebugInfo,
) -> Vec<(usize, usize)>
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    // unwrap ok because only numeric memory data will parse sucessfully
    let parse_data = |opt: Option<String>| opt.map_or(0, |s| s.parse::<usize>().unwrap());

    let mut label_mems = vec![];
    let mut raw_mems = vec![];

    for mem in mems {
        if let Ok(addr) = mem.addr.parse::<usize>() {
            raw_mems.push((addr, parse_data(mem.data)));
        } else {
            label_mems.push((mem.addr, parse_data(mem.data)));
        }
    }

    let mut used_addr = raw_mems.iter().map(|x| x.0).collect::<Vec<_>>();

    let mut links = vec![];

    for (i, (addr, _)) in label_mems.iter().enumerate() {
        for (
            j,
            Ir {
                inst: Inst { op, .. },
                ..
            },
        ) in prog.iter().enumerate()
        {
            match op {
                Op::Fail(x) => {
                    if addr == x {
                        links.push((i, j, None));
                    }
                }
                Op::MultiOp(vec) => {
                    for (idx, op) in vec.iter().enumerate() {
                        if let Op::Fail(x) = op {
                            if addr == x {
                                links.push((i, j, Some(idx)));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    debug!("Detected links between program and memory:");
    debug!("{:?}\n", links);

    let uids: Vec<_> = {
        used_addr.sort_unstable();

        let (first, last) = if used_addr.is_empty() {
            (0, 0)
        } else {
            // unwrap ok because vector is guaranteed to not be empty
            (
                used_addr.first().copied().unwrap(),
                used_addr.last().copied().unwrap(),
            )
        };

        if (0..first).len() > links.len() {
            (0..first).take(links.len()).collect()
        } else {
            (last..).take(links.len()).collect()
        }
    };

    assert!(
        uids.len() >= links.len(),
        "One of the memory addresses is too big"
    );

    let mut newlinks = BTreeMap::new();

    // linking
    for ((memaddr, progaddr, multiop_idx), uid) in links.into_iter().zip(uids) {
        let (addr, data) = &label_mems[memaddr];

        let uid = newlinks.entry(addr).or_insert((uid, *data)).0;

        debug_info.mem.entry(uid).or_insert_with(|| addr.clone());

        let cir = &mut prog[progaddr];

        match cir.inst.op {
            Op::MultiOp(ref mut ops) if multiop_idx.is_some() => {
                ops[multiop_idx.unwrap()] = Op::Addr(uid);
            }
            Op::Fail(_) => cir.inst.op = Op::Addr(uid),
            _ => {}
        }
    }

    newlinks
        .values()
        .into_iter()
        .copied()
        .chain(raw_mems)
        .collect()
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

            let mut exec = jit::<DefaultSet, _>(prog, make_io!(std::io::stdin(), s.clone()));

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
        let mut exec = jit::<DefaultSet, _>(
            include_str!("../examples/panics.pasm"),
            make_io!(std::io::stdin(), std::io::sink()),
        );
        exec.exec::<DefaultSet>();
    }
}
