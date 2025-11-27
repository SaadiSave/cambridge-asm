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

    #[allow(clippy::type_complexity)]
    pub fn parse(mut self) -> Result<(Vec<InstIr<I>>, Vec<MemIr>, DebugInfo), ErrorMap> {
        let (inst_spans, mut insts, mut mems) = self.get_insts_and_mems();

        self.debug_info.inst_spans = inst_spans;

        linker::Linker::new(&mut insts, &mut mems).link();

        if self.err.is_empty() {
            Ok((
                insts
                    .into_iter()
                    .map(InstIr::try_from)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap(),
                mems.into_iter()
                    .map(MemIr::try_from)
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap(),
                self.debug_info,
            ))
        } else {
            Err(self.err)
        }
    }
}

mod linker {
    use super::{Addr, Debug, Display, Inst, Mem, Op};
    use crate::inst::InstSet;
    use std::{
        collections::{HashMap, HashSet},
        ops::Deref,
    };

    #[derive(Debug, Clone, Copy)]
    enum Src {
        Prog(usize),
        Mem(usize),
    }

    impl From<Src> for Op {
        fn from(value: Src) -> Self {
            match value {
                Src::Mem(addr) | Src::Prog(addr) => Op::Addr(addr),
            }
        }
    }

    #[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
    enum Instance {
        MultiOp(usize, usize),
        Single(usize),
    }

    #[derive(Debug, Clone, Default)]
    struct SymbolData {
        source: Option<Src>,
        instances: HashSet<Instance>,
    }

    #[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
    enum Symbol {
        Label(&'static str),
        Addr(usize),
    }

    impl From<&Addr> for Symbol {
        fn from(value: &Addr) -> Self {
            match value {
                &Addr::Bare(addr) => Self::Addr(addr),
                Addr::Label(label) => label.into(),
            }
        }
    }

    impl<'a> From<&'a String> for Symbol {
        fn from(value: &'a String) -> Self {
            // leak it so that symbol copies are cheap
            Self::Label(Box::leak(value.clone().into_boxed_str()))
        }
    }

    impl Display for Symbol {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Symbol::Label(s) => Display::fmt(&s, f),
                Symbol::Addr(addr) => Display::fmt(&addr, f),
            }
        }
    }

    impl TryFrom<&Op> for Symbol {
        type Error = ();

        fn try_from(value: &Op) -> Result<Self, Self::Error> {
            match value {
                &Op::Addr(addr) => Ok(Symbol::Addr(addr)),
                Op::Fail(s) => Ok(s.into()),
                Op::Indirect(op) => op.deref().try_into(),
                _ => Err(()),
            }
        }
    }

    type SymbolTableInner = HashMap<Symbol, SymbolData>;

    #[derive(Debug, Clone)]
    struct SymbolTable(SymbolTableInner);

    impl IntoIterator for SymbolTable {
        type Item = (Src, HashSet<Instance>);
        type IntoIter = std::iter::Map<
            <SymbolTableInner as IntoIterator>::IntoIter,
            fn(<SymbolTableInner as IntoIterator>::Item) -> Self::Item,
        >;

        fn into_iter(self) -> Self::IntoIter {
            self.0
                .into_iter()
                .map(|(sym, SymbolData { source, instances })| {
                    (
                        source.unwrap_or_else(|| panic!("{sym} is undefined")),
                        instances,
                    )
                })
        }
    }

    impl SymbolTable {
        pub fn new() -> Self {
            Self(HashMap::new())
        }

        pub fn add_instance(&mut self, symbol: Symbol, instance: Instance) {
            self.0
                .entry(symbol)
                .and_modify(|SymbolData { instances, .. }| {
                    (instances).insert(instance);
                })
                .or_insert_with(|| {
                    let mut instances = HashSet::new();
                    instances.insert(instance);
                    SymbolData {
                        source: None,
                        instances,
                    }
                });
        }

        pub fn add_src(&mut self, symbol: Symbol, src: Src) {
            self.0
                .entry(symbol)
                .and_modify(|SymbolData { source, .. }| {
                    if source.is_some() {
                        panic!("{symbol} is defined multiple times");
                    } else {
                        *source = Some(src);
                    }
                }); // do nothing if symbol doesn't exist
        }
    }

    impl Op {
        fn link(&mut self, src: Src, mop_idx: Option<usize>) {
            match self {
                Op::Addr(_) | Op::Fail(_) => *self = src.into(),
                Op::MultiOp(ops) if mop_idx.is_some() => ops[mop_idx.unwrap()] = src.into(),
                Op::Indirect(op) => op.link(src, mop_idx),
                _ => panic!("Symbol linking failed. Please report this error.\nop: {self:?}\nsrc: {src:?}\nmop_idx: {mop_idx:?}"),
            }
        }
    }

    pub struct Linker<'inst, 'mem, I> {
        symbol_table: SymbolTable,
        used_addrs: HashSet<usize>,
        program: &'inst mut [Inst<I>],
        memory: &'mem mut [Mem],
    }

    impl<'inst, 'mem, I> Linker<'inst, 'mem, I>
    where
        I: InstSet,
        <I as std::str::FromStr>::Err: Display,
    {
        pub fn new(prog: &'inst mut [Inst<I>], mem: &'mem mut [Mem]) -> Self {
            Self {
                symbol_table: SymbolTable::new(),
                used_addrs: HashSet::new(),
                program: prog,
                memory: mem,
            }
        }

        fn find_symbols(&mut self) {
            for (idx, Inst { op, .. }) in self.program.iter().enumerate() {
                match op {
                    Op::MultiOp(ops) => {
                        for (mop_idx, sym) in ops
                            .iter()
                            .enumerate()
                            .filter_map(|(idx, op)| Symbol::try_from(op).map(|sym| (idx, sym)).ok())
                        {
                            self.symbol_table
                                .add_instance(sym, Instance::MultiOp(idx, mop_idx));
                        }
                    }
                    op => {
                        // Failure irrelevant
                        if let Ok(sym) = Symbol::try_from(op) {
                            self.symbol_table.add_instance(sym, Instance::Single(idx));
                        }
                    }
                }
            }
        }

        fn find_symbol_sources(&mut self) {
            // find which of the symbols are instruction addresses
            for (idx, Inst { addr, .. }) in self.program.iter().enumerate() {
                if let Some(addr) = addr {
                    // add_src automatically does nothing if symbol is absent
                    self.symbol_table.add_src(addr.into(), Src::Prog(idx));
                }
            }

            // leave explicit memory addresses untouched
            for addr in self.memory.iter().filter_map(|Mem { addr, .. }| {
                if let &Addr::Bare(addr) = addr {
                    Some(addr)
                } else {
                    None
                }
            }) {
                self.symbol_table
                    .add_src(Symbol::Addr(addr), Src::Mem(addr));
                assert!(
                    self.used_addrs.insert(addr),
                    "{addr:?} is used multiple times"
                );
            }
        }

        fn readdress(&mut self) {
            for (idx, Inst { addr, .. }) in self.program.iter_mut().enumerate() {
                *addr = Some(Addr::Bare(idx));
            }

            let mut counter = 0;

            for addr in self.memory.iter_mut().filter_map(|Mem { addr, .. }| {
                if matches!(addr, Addr::Label(_)) {
                    Some(addr)
                } else {
                    None
                }
            }) {
                // find unused address
                while self.used_addrs.contains(&counter) {
                    counter += 1;
                }
                self.symbol_table
                    .add_src(Symbol::from(&addr.clone()), Src::Mem(counter));

                *addr = Addr::Bare(counter);

                counter += 1;
            }
        }

        pub fn link(mut self) {
            self.find_symbols();
            self.find_symbol_sources();
            self.readdress();

            for (src, instances) in self.symbol_table {
                for instance in instances {
                    match instance {
                        Instance::MultiOp(idx, mop_idx) => {
                            self.program[idx].op.link(src, Some(mop_idx));
                        }
                        Instance::Single(idx) => self.program[idx].op.link(src, None),
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Addr {
    Bare(usize),
    Label(String),
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

impl<I> TryFrom<Inst<I>> for InstIr<I>
where
    I: InstSet,
    <I as FromStr>::Err: Display,
{
    type Error = ();

    fn try_from(Inst { addr, opcode, op }: Inst<I>) -> Result<Self, Self::Error> {
        if let Some(Addr::Bare(addr)) = addr {
            Ok(Self::new(addr, opcode, op))
        } else {
            Err(())
        }
    }
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

impl TryFrom<Mem> for MemIr {
    type Error = ();

    fn try_from(Mem { addr, data }: Mem) -> Result<Self, Self::Error> {
        if let Addr::Bare(addr) = addr {
            Ok(Self { addr, data })
        } else {
            Err(())
        }
    }
}
