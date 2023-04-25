// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    exec::{self, DebugInfo},
    inst::{self, InstSet, Op},
    parse::lexer::{ErrorKind, ErrorMap, ParseError, Token, TokensWithSpan, WithSpan},
};
use logos::Logos;
use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::Range,
    str::FromStr,
};

macro_rules! store_err {
    ($store:expr, $span:expr, $err:expr) => {
        $store.entry($span).or_insert($err)
    };
}

type Line = Vec<WithSpan<Token>>;

#[derive(Clone)]
pub struct Parser<'a, I> {
    pub src: &'a str,
    lines: Vec<Line>,
    err: ErrorMap,
    debug_info: DebugInfo,
    _inst_set: PhantomData<I>,
}

impl<'a, I> Parser<'a, I>
where
    I: InstSet,
    <I as FromStr>::Err: Display,
{
    pub fn new(src: &'a str) -> Self {
        let (lines, err) = TokensWithSpan(Token::lexer(src)).lines();
        Self {
            src,
            lines,
            err,
            debug_info: DebugInfo::default(),
            _inst_set: PhantomData,
        }
    }

    fn get_inst(line: &[WithSpan<Token>]) -> Result<Option<Inst<I>>, ParseError> {
        let rawline = line.iter().map(|(_, t)| t).cloned().collect::<Vec<_>>();

        let (addr, (opcodeidx, opcode), (restidx, rest)) = match rawline.as_slice() {
            [Token::BareNumber(addr), Token::Text(opcode), rest @ ..] => {
                (Some(Addr::Bare(*addr)), (1, opcode), (2, rest))
            }
            [Token::Text(label), Token::Colon, Token::Text(opcode), rest @ ..] => {
                (Some(Addr::Label(label.clone())), (2, opcode), (3, rest))
            }
            [Token::Text(opcode), rest @ ..] => (None, (0, opcode), (1, rest)),
            [] => return Ok(None),
            _ => {
                let ((s, _), (e, _)) = (line.first().unwrap(), line.last().unwrap());

                return Err((s.start..e.end, ErrorKind::SyntaxError));
            }
        };

        let opcode = match I::from_str(opcode) {
            Ok(s) => s,
            Err(e) => {
                let span = line[opcodeidx].0.clone();

                return Err((span, ErrorKind::InvalidOpcode(e.to_string())));
            }
        };

        if let Some((idx, _)) = rest.iter().enumerate().find(|(_, t)| {
            !matches!(
                t,
                Token::Gpr(_)
                    | Token::BareNumber(_)
                    | Token::Text(_)
                    | Token::Comma
                    | Token::Literal(_)
            )
        }) {
            let span = line[restidx + idx].0.clone();

            return Err((span, ErrorKind::InvalidOperand));
        }

        let mut ops = rest
            .iter()
            .cloned()
            .filter(|t| !matches!(t, Token::Comma))
            .map(Op::from)
            .collect::<Vec<_>>();

        let op = match ops.len() {
            0 => Op::Null,
            1 => ops.pop().unwrap(),
            _ => Op::MultiOp(ops),
        };

        debug!(
            "{:>4}\t{:>4}\t{:<4}",
            addr.as_ref().map(ToString::to_string).unwrap_or_default(),
            opcode,
            op,
        );

        Ok(Some(Inst { addr, opcode, op }))
    }

    fn get_mem(line: &[WithSpan<Token>]) -> Result<Option<Mem>, ParseError> {
        let rawline = line.iter().map(|(_, t)| t).cloned().collect::<Vec<_>>();

        let (&Range { start, .. }, &Range { end, .. }) = if rawline.is_empty() {
            return Ok(None);
        } else {
            (&line.first().unwrap().0, &line.last().unwrap().0)
        };

        let get_data = |t: &[Token], start_idx: usize| -> Result<usize, ParseError> {
            match t {
                &[Token::BareNumber(n)] => Ok(n),
                [] => Ok(0),
                _ => Err((line[start_idx].0.start..end, ErrorKind::SyntaxError)),
            }
        };

        match rawline.as_slice() {
            &[Token::BareNumber(addr), ref rest @ ..] => Ok(Some(Mem {
                addr: Addr::Bare(addr),
                data: get_data(rest, 1)?,
            })),
            [Token::Text(label), Token::Colon, rest @ ..] => Ok(Some(Mem {
                addr: Addr::Label(label.clone()),
                data: get_data(rest, 2)?,
            })),
            [] => Ok(None),
            _ => Err((start..end, ErrorKind::SyntaxError)),
        }
    }

    fn get_insts_and_mems(&mut self) -> (Vec<Inst<I>>, Vec<Mem>) {
        let mut blocks = self
            .lines
            .split(Vec::is_empty)
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();

        assert!((blocks.len() >= 2), "Unable to parse. Your source may not contain blank line(s) between the program and the memory, or the memory might be absent");

        let mems = blocks
            .pop()
            .unwrap()
            .iter()
            .map(|line| Self::get_mem(line))
            .filter_map(|res| match res {
                Ok(mem @ Some(_)) => mem,
                Ok(None) => None,
                Err((span, err)) => {
                    store_err!(self.err, span, err);
                    None
                }
            })
            .collect::<Vec<_>>();

        let insts = blocks
            .concat()
            .iter()
            .map(|line| Self::get_inst(line))
            .filter_map(|res| match res {
                Ok(inst @ Some(_)) => inst,
                Ok(None) => None,
                Err((span, err)) => {
                    store_err!(self.err, span, err);
                    None
                }
            })
            .collect::<Vec<_>>();

        (insts, mems)
    }

    fn process_insts(&mut self, insts: Vec<Inst<I>>) -> Vec<InstIr<I>> {
        fn op_addr_eq(op: &Op, addr: &Addr) -> bool {
            match (op, addr) {
                (Op::Addr(x), Addr::Bare(bare)) => x == bare,
                (Op::Fail(x), Addr::Label(label)) => x == label,
                _ => false,
            }
        }

        self.debug_info
            .prog
            .extend(
                insts
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, Inst { addr, .. })| {
                        addr.as_ref().map(|a| (idx, a.as_dbg_string()))
                    }),
            );

        let mut links = Vec::new();

        for (i, Inst { addr, .. }) in insts.iter().enumerate() {
            for (j, Inst { op, .. }) in insts.iter().enumerate() {
                if let Some(addr) = addr {
                    match op {
                        Op::MultiOp(vec) => {
                            for (idx, op) in vec.iter().enumerate() {
                                if op_addr_eq(op, addr) {
                                    links.push((i, j, Some(idx)));
                                }
                            }
                        }
                        _ => {
                            if op_addr_eq(op, addr) {
                                links.push((i, j, None));
                            }
                        }
                    }
                }
            }
        }

        let mut ir = insts
            .into_iter()
            .enumerate()
            .map(|(idx, inst)| (idx, inst))
            .collect::<Vec<_>>();

        for (to, from, multiop_idx) in links {
            match &ir[from].1.op {
                Op::MultiOp(ops) => {
                    let mut ops = ops.clone();
                    // unwrap ok because mutiop_idx will always exist if operand is multiop
                    ops[multiop_idx.unwrap()] = Op::Addr(to);
                    ir[from].1.op = Op::MultiOp(ops);
                }
                Op::Addr(_) | Op::Fail(_) => ir[from].1.op = Op::Addr(to),
                _ => {}
            };
        }

        ir.into_iter()
            .map(|(idx, Inst { opcode, op, .. })| InstIr::new(idx, opcode, op))
            .collect()
    }

    fn process_mems(&mut self, mems: Vec<Mem>, prog: &mut [InstIr<I>]) -> Vec<MemIr> {
        let mut label_mems = Vec::new();
        let mut raw_mems = Vec::new();

        for Mem { addr, data } in mems {
            match addr {
                Addr::Bare(bare) => raw_mems.push((bare, data)),
                Addr::Label(label) => label_mems.push((label, data)),
            }
        }

        let mut links = vec![];

        for (i, (addr, _)) in label_mems.iter().enumerate() {
            for (
                j,
                InstIr {
                    inst: inst::Inst { op, .. },
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

        let unused_addrs: Vec<_> = {
            let mut used_addr = raw_mems.iter().map(|x| x.0).collect::<Vec<_>>();

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

            (0..first).chain(last + 1..).take(links.len()).collect()
        };

        assert!(
            unused_addrs.len() >= links.len(),
            "One of the memory addresses is too big"
        );

        let mut newlinks = BTreeMap::new();

        // linking
        for ((memaddr, progaddr, multiop_idx), uid) in links.into_iter().zip(unused_addrs) {
            let (addr, data) = &label_mems[memaddr];

            let uid = newlinks.entry(addr).or_insert((uid, *data)).0;

            self.debug_info
                .mem
                .entry(uid)
                .or_insert_with(|| addr.clone());

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
            .copied()
            .chain(raw_mems)
            .map(|(addr, data)| MemIr { addr, data })
            .collect()
    }

    #[allow(clippy::type_complexity)]
    pub fn parse(mut self) -> Result<(Vec<InstIr<I>>, Vec<MemIr>, DebugInfo), ErrorMap> {
        let (insts, mems) = self.get_insts_and_mems();

        let mut inst_ir = self.process_insts(insts);

        let mem_ir = self.process_mems(mems, &mut inst_ir);

        if self.err.is_empty() {
            Ok((inst_ir, mem_ir, self.debug_info))
        } else {
            Err(self.err)
        }
    }
}

#[derive(Debug, Clone)]
pub enum Addr {
    Bare(usize),
    Label(String),
}

impl Addr {
    fn as_dbg_string(&self) -> String {
        match self {
            Addr::Label(label) => label.clone(),
            Addr::Bare(bare) => bare.to_string(),
        }
    }
}

impl Display for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bare(addr) => write!(f, "{addr}"),
            Self::Label(label) => write!(f, "{label}:"),
        }
    }
}

pub struct InstIr<I>
where
    I: InstSet,
    <I as FromStr>::Err: Display,
{
    pub addr: usize,
    pub inst: inst::Inst<I>,
}

impl<I> InstIr<I>
where
    I: InstSet,
    <I as FromStr>::Err: Display,
{
    pub fn new(addr: usize, opcode: I, op: Op) -> Self {
        Self {
            addr,
            inst: inst::Inst { inst: opcode, op },
        }
    }
}

impl<I> From<InstIr<I>> for (usize, exec::ExecInst)
where
    I: InstSet,
    <I as FromStr>::Err: Display,
{
    fn from(InstIr { addr, inst }: InstIr<I>) -> Self {
        (addr, inst.to_exec_inst())
    }
}

pub struct Inst<I> {
    pub addr: Option<Addr>,
    pub opcode: I,
    pub op: Op,
}

impl<I> Debug for Inst<I>
where
    I: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inst")
            .field("addr", &self.addr)
            .field("opcode", &self.opcode.to_string())
            .field("op", &self.op)
            .finish()
    }
}

impl<I> Display for Inst<I>
where
    I: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.addr
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            self.opcode,
            self.op
        )
    }
}

pub struct Mem {
    pub addr: Addr,
    pub data: usize,
}

pub struct MemIr {
    pub addr: usize,
    pub data: usize,
}
