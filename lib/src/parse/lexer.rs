// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::inst::Op;
use logos::{Lexer, Logos};
use std::{collections::HashMap, fmt::Debug, num::ParseIntError, ops::Range};
use thiserror::Error;

fn parse_num(lex: &mut Lexer<Token>) -> Result<usize, ErrorKind> {
    let src = if lex.slice().as_bytes()[0] == b'#' {
        &lex.slice()[1..]
    } else {
        lex.slice()
    };

    let res = match src.as_bytes()[0] {
        b'b' | b'B' => usize::from_str_radix(&src[1..], 2),
        b'x' | b'X' | b'&' => usize::from_str_radix(&src[1..], 16),
        b'o' | b'O' => usize::from_str_radix(&src[1..], 8),
        _ => src.parse(),
    }?;

    Ok(res)
}

fn pop_parens(lex: &mut Lexer<Token>) -> String {
    let mut chars = lex.slice().chars();
    chars.next();
    chars.next_back();
    chars.collect()
}

#[derive(Default, Error, Debug, Clone, PartialEq)]
pub enum ErrorKind {
    #[error("Invalid integer format")]
    ParseIntError(#[from] ParseIntError),
    #[error("Syntax error")]
    #[default]
    SyntaxError,
    #[error("Invalid opcode `{0}`")]
    InvalidOpcode(String),
    #[error("Invalid operand")]
    InvalidOperand,
}

pub type ErrorMap = HashMap<Span, ErrorKind>;

pub type ParseError = WithSpan<ErrorKind>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinearMemory {
    pub init: usize,
    pub len: usize,
}

impl LinearMemory {
    pub(self) fn from_lexer(lexer: &mut Lexer<Token>) -> Self {
        Self::from_str(lexer.slice())
    }

    pub(self) fn from_str(s: &str) -> Self {
        let mut decl = s.trim_matches(|c| c == '[' || c == ']').split(';');

        let init = decl.next().unwrap().parse().unwrap();
        let len = decl.next().unwrap().parse().unwrap();

        Self { init, len }
    }
}

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(skip r"[ \t]")]
#[logos(error = ErrorKind)]
pub enum Token {
    #[regex(r"//[^\r\n]*", logos::skip)]
    Comment,

    #[regex(r"\w*", |lex| lex.slice().to_string(), priority = 0)]
    Text(String),

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[regex("r[0-9][0-9]?", |lex| lex.slice()[1..].parse())]
    Gpr(usize),

    #[regex("#[&xXoObB][0-9a-fA-F]+", parse_num)]
    #[regex("#[0-9]+", parse_num)]
    Literal(usize),

    #[regex("[xXoObB][0-9a-fA-F]+", parse_num)]
    #[regex("[0-9]+", parse_num)]
    BareNumber(usize),

    #[regex(r"\(\w*\)", pop_parens)]
    Indirect(String),

    #[regex(r"(?:\r\n)|\n")]
    Newline,

    #[regex(r"\[[0-9]+;[0-9]+\]", LinearMemory::from_lexer)]
    LinearMemory(LinearMemory),
}

impl From<Token> for Op {
    fn from(t: Token) -> Self {
        match t {
            Token::BareNumber(addr) => Op::Addr(addr),
            Token::Gpr(r) => Op::Gpr(r),
            Token::Literal(lit) => Op::Literal(lit),
            Token::Text(txt) => match txt.to_lowercase().as_str() {
                "acc" => Op::Acc,
                "cmp" => Op::Cmp,
                "ix" => Op::Ix,
                "ar" => Op::Ar,
                _ => Op::Fail(txt),
            },
            Token::Indirect(s) => Op::Indirect(Box::new(Op::from(s))),
            _ => unreachable!(),
        }
    }
}

pub type Span = Range<usize>;

pub type WithSpan<T> = (Span, T);

#[derive(Debug, Clone)]
pub struct TokensWithError<'a>(pub Lexer<'a, Token>);

impl TokensWithError<'_> {
    pub fn lines(mut self) -> (Vec<Vec<WithSpan<Token>>>, ErrorMap) {
        let mut errors = ErrorMap::new();
        let acc = self.by_ref().fold(vec![Vec::new()], |mut acc, (r, t)| {
            match t {
                Ok(Token::Newline) => {
                    acc.push(Vec::new());
                }
                Ok(t) => {
                    acc.last_mut().unwrap().push((r, t));
                }
                Err(e) => {
                    errors.entry(r).or_insert(e);
                }
            }

            acc
        });

        (acc, errors)
    }
}

impl Iterator for TokensWithError<'_> {
    type Item = WithSpan<Result<Token, ErrorKind>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|token| (self.0.span(), token))
    }
}
