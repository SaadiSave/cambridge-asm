// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    exec::{self, DebugInfo},
    inst::{self, InstSet, Op},
    parse::lexer::{
        ErrorKind, ErrorMap, LinearMemory, ParseError, Span, Token, TokensWithError, WithSpan,
    },
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
    #[allow(dead_code)]
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
        let (lines, err) = TokensWithError(Token::lexer(src)).lines();
        Self {
            src,
            lines,
            err,
            debug_info: DebugInfo::default(),
            _inst_set: PhantomData,
        }
    }

    fn get_inst(line: &[WithSpan<Token>]) -> Result<Option<WithSpan<Inst<I>>>, ParseError> {
        let span = {
            let ((s, _), (e, _)) = (line.first().unwrap(), line.last().unwrap());
            s.start..e.end
        };

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
                return Err((span, ErrorKind::SyntaxError));
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
                    | Token::Indirect(_)
            )
        }) {
            let span = line[restidx + idx].0.clone();

            return Err((span, ErrorKind::InvalidOperand));
        }

        let mut ops = rest
            .iter()
            .filter(|t| !matches!(t, Token::Comma))
            .cloned()
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

        Ok(Some((span, Inst { addr, opcode, op })))
    }

    fn get_mem(line: &[WithSpan<Token>]) -> Result<Option<MemEnum>, ParseError> {
        enum DataEnum {
            LinearMemory(LinearMemory),
            Normal(usize),
        }

        let rawline = line.iter().map(|(_, t)| t).cloned().collect::<Vec<_>>();

        let (&Range { start, .. }, &Range { end, .. }) = if rawline.is_empty() {
            return Ok(None);
        } else {
            (&line.first().unwrap().0, &line.last().unwrap().0)
        };

        let get_data = |t: &[Token], start_idx: usize| -> Result<DataEnum, ParseError> {
            match t {
                &[Token::BareNumber(n)] => Ok(DataEnum::Normal(n)),
                &[Token::LinearMemory(mem)] => Ok(DataEnum::LinearMemory(mem)),
                [] => Ok(DataEnum::Normal(0)),
                _ => Err((line[start_idx].0.start..end, ErrorKind::SyntaxError)),
            }
        };

        match rawline.as_slice() {
            &[Token::BareNumber(addr), ref rest @ ..] => {
                let res = match get_data(rest, 1)? {
                    DataEnum::LinearMemory(mem) => Some(MemEnum::Linear(
                        (addr..addr + mem.len)
                            .map(Addr::Bare)
                            .map(move |addr| (addr, mem.init))
                            .map(Mem::from)
                            .collect(),
                    )),
                    DataEnum::Normal(data) => Some(MemEnum::One(Mem {
                        addr: Addr::Bare(addr),
                        data,
                    })),
                };

                Ok(res)
            }
            [Token::Text(label), Token::Colon, rest @ ..] => Ok(Some(MemEnum::One(Mem {
                addr: Addr::Label(label.clone()),
                data: match get_data(rest, 2)? {
                    DataEnum::LinearMemory(_) => Err((start..end, ErrorKind::SyntaxError))?,
                    DataEnum::Normal(data) => data,
                },
            }))),
            [] => Ok(None),
            _ => Err((start..end, ErrorKind::SyntaxError)),
        }
    }

    fn get_insts_and_mems(&mut self) -> (Vec<Span>, Vec<Inst<I>>, Vec<Mem>) {
        let mut blocks = self
            .lines
            .split(Vec::is_empty)
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();

        assert!(blocks.len() >= 2, "Unable to parse. Your source may not contain blank line(s) between the program and the memory, or the memory might be absent");

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
            .fold(Vec::new(), |mut acc, mem| {
                match mem {
                    MemEnum::Linear(mems) => acc.extend(mems),
                    MemEnum::One(mem) => acc.push(mem),
                }

                acc
            });

        let (inst_spans, insts): (Vec<_>, Vec<_>) = blocks
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
            .unzip();

        (inst_spans, insts, mems)
    }

    fn process_insts(&mut self, insts: Vec<Inst<I>>) -> Vec<InstIr<I>> {
        fn op_addr_eq(op: &Op, addr: &Addr) -> bool {
            match (op, addr) {
                (Op::Addr(x), Addr::Bare(bare)) => x == bare,
                (Op::Fail(x), Addr::Label(label)) => x == label,
                (Op::Indirect(op), addr) => op_addr_eq(op.as_ref(), addr),
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

        let mut ir = insts.into_iter().enumerate().collect::<Vec<_>>();

        for (to, from, multiop_idx) in links {
            match &mut ir[from].1.op {
                Op::MultiOp(ops) => {
                    for (idx, op) in ops.iter_mut().enumerate() {
                        if idx == multiop_idx.unwrap() {
                            match op {
                                Op::Addr(_) | Op::Fail(_) => *op = Op::Addr(to),
                                Op::Indirect(op) => {
                                    if matches!(op.as_ref(), Op::Addr(_) | Op::Fail(_)) {
                                        *op.as_mut() = Op::Addr(to);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Op::Addr(_) | Op::Fail(_) => ir[from].1.op = Op::Addr(to),
                Op::Indirect(op) => {
                    if matches!(op.as_ref(), Op::Addr(_) | Op::Fail(_)) {
                        *op.as_mut() = Op::Addr(to);
                    }
                }
                _ => {}
            }
        }

        ir.into_iter()
            .map(|(idx, Inst { opcode, op, .. })| InstIr::new(idx, opcode, op))
            .collect()
    }

    fn process_mems(&mut self, mems: Vec<Mem>, prog: &mut [InstIr<I>]) -> Vec<MemIr> {
        fn op_label_eq(op: &Op, label: &str) -> bool {
            match op {
                Op::Fail(x) => x == label,
                Op::Indirect(op) => op_label_eq(op.as_ref(), label),
                _ => false,
            }
        }

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
                    Op::MultiOp(vec) => {
                        for (idx, op) in vec.iter().enumerate() {
                            if op_label_eq(op, addr) {
                                links.push((i, j, Some(idx)));
                            }
                        }
                    }
                    _ => {
                        if op_label_eq(op, addr) {
                            links.push((i, j, None));
                        }
                    }
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
                Op::MultiOp(ref mut ops) => {
                    for (idx, op) in ops.iter_mut().enumerate() {
                        if idx == multiop_idx.unwrap() {
                            match op {
                                Op::Fail(_) => *op = Op::Addr(uid),
                                Op::Indirect(_) => {
                                    if let Op::Indirect(op) = op {
                                        if matches!(op.as_ref(), Op::Fail(_)) {
                                            *op.as_mut() = Op::Addr(uid);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                ref mut op @ Op::Fail(_) => *op = Op::Addr(uid),
                Op::Indirect(ref mut op) => {
                    if matches!(op.as_ref(), Op::Fail(_)) {
                        *op.as_mut() = Op::Addr(uid);
                    }
                }
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
        let (inst_spans, insts, mems) = self.get_insts_and_mems();

        self.debug_info.inst_spans = inst_spans;

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
            inst: inst::Inst::new(opcode, op),
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

enum MemEnum {
    Linear(Vec<Mem>),
    One(Mem),
}

pub struct Mem {
    pub addr: Addr,
    pub data: usize,
}

impl From<(Addr, usize)> for Mem {
    fn from((addr, data): (Addr, usize)) -> Self {
        Self { addr, data }
    }
}

pub struct MemIr {
    pub addr: usize,
    pub data: usize,
}
