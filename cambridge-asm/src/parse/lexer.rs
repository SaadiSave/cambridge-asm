// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::inst::Op;
use logos::{Lexer, Logos};
use std::{collections::HashMap, fmt::Debug, num::ParseIntError, ops::Range};

fn parse_num(lex: &mut Lexer<Token>) -> Option<usize> {
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
    };

    res.map_err(|e| {
        lex.extras
            .push_error(lex.span(), ErrorKind::ParseIntError(e))
    })
    .ok()
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    ParseIntError(ParseIntError),
    SyntaxError,
    InvalidOpcode(String),
    InvalidOperand,
}

pub type ErrorMap = HashMap<Span, ErrorKind>;

pub type ParseError = WithSpan<ErrorKind>;

#[derive(Default, Debug, Clone)]
pub struct Extras {
    pub errors: ErrorMap,
}

impl Extras {
    pub fn push_error(&mut self, span: Range<usize>, err: ErrorKind) -> &mut ErrorKind {
        self.errors.entry(span).or_insert(err)
    }
}

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
#[logos(extras = Extras)]
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

    #[regex("#[xXoObB][0-9a-fA-F]+", parse_num)]
    #[regex("#[0-9]+", parse_num)]
    Literal(usize),

    #[regex("[xXoObB][0-9a-fA-F]+", parse_num)]
    #[regex("[0-9]+", parse_num)]
    BareNumber(usize),

    #[regex(r"[ \t]", logos::skip)]
    Whitespace,

    #[regex(r"(?:\r\n)|\n")]
    Newline,

    #[error]
    Error,
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
            _ => unreachable!(),
        }
    }
}

pub type Span = Range<usize>;

pub type WithSpan<T> = (Span, T);

#[derive(Debug, Clone)]
pub struct TokensWithSpan<'a>(pub Lexer<'a, Token>);

impl<'a> TokensWithSpan<'a> {
    pub fn lines(mut self) -> (Vec<Vec<WithSpan<Token>>>, ErrorMap) {
        let acc = self.by_ref().fold(vec![Vec::new()], |mut acc, (r, t)| {
            if matches!(t, Token::Newline) {
                acc.push(Vec::new());
            } else {
                acc.last_mut().unwrap().push((r, t));
            }

            acc
        });

        let errs = self.0.extras.errors;

        (acc, errs)
    }
}

impl<'a> Iterator for TokensWithSpan<'a> {
    type Item = WithSpan<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|token| (self.0.span(), token))
    }
}
