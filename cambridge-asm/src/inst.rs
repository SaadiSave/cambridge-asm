// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(clippy::module_name_repetitions)]

use crate::exec::{Context, ExecFunc, ExecInst, PasmError};
use std::{fmt::Display, ops::Deref, str::FromStr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "bincode")]
use bincode::{Decode, Encode};

#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bincode", derive(Encode, Decode))]
pub enum Op {
    Fail(String),
    Acc,
    Ix,
    Cmp,
    Ar,
    Addr(usize),
    Literal(usize),
    // Prepare for gpr feature
    Gpr(usize),
    MultiOp(Vec<Op>),
    Null,
}

impl Op {
    // pub fn map_res<T, E>(&self, f: impl Fn(&Op) -> Result<T, E>) -> Result<T, E> {
    //     f(self)
    // }
    //
    // pub fn map<O>(&self, f: impl Fn(&Op) -> O) -> O {
    //     f(self)
    // }

    pub fn is_none(&self) -> bool {
        matches!(self, Op::Null)
    }

    pub fn is_register(&self) -> bool {
        matches!(self, Op::Acc | Op::Ix | Op::Ar | Op::Gpr(_))
    }

    pub fn is_read_write(&self) -> bool {
        self.is_register() || matches!(self, Op::Addr(_))
    }

    pub fn is_usizeable(&self) -> bool {
        self.is_read_write() || matches!(self, Op::Literal(_))
    }

    /// will panic if [`Op::is_usizeable`] is not checked first
    pub fn get_val(&self, ctx: &Context) -> Result<usize, PasmError> {
        match self {
            &Op::Literal(val) => Ok(val),
            Op::Addr(addr) => ctx.mem.get(addr),
            reg if reg.is_register() => Ok(ctx.get_register(reg)),
            _ => unreachable!(),
        }
    }
}

impl ToString for Op {
    fn to_string(&self) -> String {
        #[allow(clippy::enum_glob_use)]
        use Op::*;

        match self {
            Null => "None".to_string(),
            Acc => "ACC".to_string(),
            Ix => "IX".to_string(),
            Cmp => "CMP".to_string(),
            Ar => "AR".to_string(),
            Addr(x) => format!("{x}"),
            Literal(x) => format!("#{x}"),
            Fail(x) => format!("`{x}` was not parsed successfully"),
            Gpr(x) => format!("r{x}"),
            MultiOp(v) => v.iter().enumerate().fold(String::new(), |out, (idx, op)| {
                let op = op.to_string();
                if idx == v.len() - 1 {
                    format!("{out}{op}")
                } else {
                    format!("{out}{op},")
                }
            }),
        }
    }
}

impl From<Op> for String {
    fn from(op: Op) -> Self {
        op.to_string()
    }
}

pub fn get_literal(mut op: String) -> usize {
    if op.starts_with('#') {
        op.remove(0);

        match op.chars().next().unwrap() {
            'b' | 'B' => {
                op.remove(0);
                usize::from_str_radix(&op, 2).unwrap()
            }
            'x' | 'X' => {
                op.remove(0);
                usize::from_str_radix(&op, 16).unwrap()
            }
            'o' | 'O' => {
                op.remove(0);
                usize::from_str_radix(&op, 8).unwrap()
            }
            '0'..='9' => op.parse().unwrap(),
            _ => unreachable!(),
        }
    } else {
        panic!("Literal `{op}` is invalid")
    }
}

pub fn get_reg_no(mut op: String) -> usize {
    op = op.to_lowercase();
    op.remove(0);

    // Ensured by parser
    op.parse().unwrap()
}

impl<T: Deref<Target = str>> From<T> for Op {
    fn from(inp: T) -> Self {
        fn get_op(inp: &str) -> Op {
            #[allow(clippy::enum_glob_use)]
            use Op::*;

            if inp.is_empty() {
                Null
            } else if let Ok(x) = inp.parse() {
                Addr(x)
            } else if inp.contains('#') {
                Literal(get_literal(inp.into()))
            } else if inp.to_lowercase().starts_with('r')
                && inp.trim_start_matches('r').chars().all(char::is_numeric)
            {
                let x = get_reg_no(inp.into());

                if x > 29 {
                    panic!("Only registers from r0 to r29 are allowed")
                } else {
                    Gpr(x)
                }
            } else {
                match inp.to_lowercase().as_str() {
                    "acc" => Acc,
                    "cmp" => Cmp,
                    "ix" => Ix,
                    _ => Fail(inp.into()),
                }
            }
        }

        if inp.contains(',') {
            Op::MultiOp(inp.split(',').map(get_op).collect())
        } else {
            get_op(&inp)
        }
    }
}

pub trait InstSet: FromStr + ToString
where
    <Self as FromStr>::Err: Display,
{
    fn as_func_ptr(&self) -> ExecFunc;
    fn from_func_ptr(_: ExecFunc) -> Result<Self, <Self as FromStr>::Err>;
}

#[macro_export]
macro_rules! inst_set {
    ($(#[$outer:meta])* $vis:vis $name:ident { $( $inst:ident => $func:expr,)+ }) => {
        inst_set! { $(#[$outer])* $vis $name use std; { $( $inst => $func,)+ } }
    };
    ($(#[$outer:meta])* $vis:vis $name:ident $using:item { $( $inst:ident => $func:expr,)+ }) => {
        $(#[$outer])*
        $vis enum $name {
            $($inst,)+
        }

        $(#[$outer])*
        impl std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_uppercase().as_str() {
                    $( stringify!($inst) => Ok(Self::$inst),)+
                    _ => Err(format!("{s} is not an operation")),
                }
            }
        }

        $(#[$outer])*
        impl ToString for $name {
            fn to_string(&self) -> String {
                match self {
                    $(Self::$inst => stringify!($inst).into(),)+
                }
            }
        }

        $(#[$outer])*
        impl $crate::inst::InstSet for $name {
            fn as_func_ptr(&self) -> $crate::exec::ExecFunc {
                $using
                match self {
                    $(Self::$inst => $func,)+
                }
            }

            fn from_func_ptr(f: $crate::exec::ExecFunc) -> Result<Self, String> {
                $using
                $(
                const $inst: $crate::exec::ExecFunc = $func;
                )+

                match f {
                    $($inst => Ok(Self::$inst),)+
                    _ => Err(format!("0x{:X} is not a valid function pointer", f as usize)),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! extend {
    ($(#[$outer:meta])* $vis:vis $name:ident extends $parent:ident { $( $inst:ident => $func:expr,)+ }) => {
        extend! { $(#[$outer])* $vis $name extends $parent use std; { $( $inst => $func,)+ } }
    };
    ($(#[$outer:meta])* $vis:vis $name:ident extends $parent:ident $using:item { $( $inst:ident => $func:expr,)+ }) => {
        $(#[$outer])*
        $vis enum $name {
            $($inst,)+
            Parent($parent)
        }

        $(#[$outer])*
        impl std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_uppercase().as_str() {
                    $( stringify!($inst) => Ok(Self::$inst),)+
                    s => Ok(Self::Parent(s.parse::<Core>()?)),
                }
            }
        }

        $(#[$outer])*
        impl ToString for $name {
            fn to_string(&self) -> String {
                match self {
                    $(Self::$inst => stringify!($inst).into(),)+
                    Self::Parent(p) => p.to_string(),
                }
            }
        }

        $(#[$outer])*
        impl $crate::inst::InstSet for $name {
            fn as_func_ptr(&self) -> $crate::exec::ExecFunc {
                $using
                match self {
                    $(Self::$inst => $func,)+
                    Self::Parent(p) => p.as_func_ptr()
                }
            }

            fn from_func_ptr(f: $crate::exec::ExecFunc) -> Result<Self, String> {
                $using
                $(
                const $inst: $crate::exec::ExecFunc = $func;
                )+

                match f {
                    $($inst => Ok(Self::$inst),)+
                    f => Ok(Self::Parent($parent::from_func_ptr(f)?)),
                }
            }
        }
    };
}

pub struct Inst<T>
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    pub inst: T,
    pub op: Op,
}

impl<T> Inst<T>
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    pub fn new(inst: T, op: Op) -> Self {
        Self { inst, op }
    }

    pub fn to_exec_inst(self) -> ExecInst {
        ExecInst::new(self.inst.as_func_ptr(), self.op)
    }
}
