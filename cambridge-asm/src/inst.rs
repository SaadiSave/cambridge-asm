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

/// Represents all possible types of pseudoassembly operands
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
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
    Gpr(usize),
    MultiOp(Vec<Op>),
    Null,
}

impl Op {
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

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::enum_glob_use)]
        use Op::*;

        let s = match self {
            Null => "None".into(),
            Acc => "ACC".into(),
            Ix => "IX".into(),
            Cmp => "CMP".into(),
            Ar => "AR".into(),
            Addr(x) => format!("{x}"),
            Literal(x) => format!("#{x}"),
            Fail(x) => format!("`{x}` was not parsed successfully"),
            Gpr(x) => format!("r{x}"),
            MultiOp(v) => v
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        };

        f.write_str(&s)
    }
}

impl From<Op> for String {
    fn from(op: Op) -> Self {
        op.to_string()
    }
}

fn get_literal(mut op: String) -> usize {
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

fn get_reg_no(mut op: String) -> usize {
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

/// Trait for instruction sets
///
/// Implement this for custom instruction sets. Manual implementation is tedious,
/// so use [`inst_set`] or [`extend`] macros if possible
pub trait InstSet: FromStr + Display
where
    <Self as FromStr>::Err: Display,
{
    fn as_func_ptr(&self) -> ExecFunc;
    fn from_func_ptr(_: ExecFunc) -> Result<Self, <Self as FromStr>::Err>;
}

/// Macro to generate an instruction set
///
/// For an example, go to this [file](https://github.com/SaadiSave/cambridge-asm/blob/main/cambridge-asm/tests/int_test.rs)
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
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(match self {
                    $(Self::$inst => stringify!($inst),)+
                })
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

/// Macro to extend an instruction set
///
/// For an example, go to this [file](https://github.com/SaadiSave/cambridge-asm/blob/main/cambridge-asm/tests/int_test.rs)
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
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$inst => f.write_str(stringify!($inst)),)+
                    Self::Parent(p) => write!(f, "{}", p),
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

/// Post-parsing representation of an instruction
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
