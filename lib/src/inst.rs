// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![allow(clippy::module_name_repetitions)]

use crate::exec::{ExecFunc, ExecInst};
use std::{fmt::Display, ops::Deref, str::FromStr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Represents all possible types of pseudoassembly operands
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Op {
    Fail(String),
    Acc,
    Ix,
    Cmp,
    Ar,
    Indirect(Box<Op>),
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
        match self {
            Op::Indirect(op) if op.is_register() => true,
            _ => matches!(self, Op::Acc | Op::Ix | Op::Ar | Op::Gpr(_)),
        }
    }

    pub fn is_read_write(&self) -> bool {
        self.is_register()
            || match self {
                Op::Indirect(op) if op.is_read_write() => true,
                _ => matches!(self, Op::Addr(_)),
            }
    }

    pub fn is_usizeable(&self) -> bool {
        self.is_read_write() || matches!(self, Op::Literal(_))
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[allow(clippy::enum_glob_use)]
        use Op::*;

        let s = match self {
            Null => String::new(),
            Acc => "ACC".into(),
            Ix => "IX".into(),
            Cmp => "CMP".into(),
            Ar => "AR".into(),
            Addr(x) => format!("{x}"),
            Literal(x) => format!("#{x}"),
            Indirect(op) => format!("({op})"),
            Fail(x) => x.clone(),
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
    fn id(&self) -> u64;
    fn from_id(_: u64) -> Result<Self, <Self as FromStr>::Err>;
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
        #[repr(u64)]
        #[derive(Clone, Copy)]
        $vis enum $name {
            $($inst,)+
        }

        $(#[$outer])*
        impl std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_uppercase().as_str() {
                    $( stringify!($inst) => Ok(Self::$inst),)+
                    _ => Err(format!("{s} is not an instruction")),
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

            fn id(&self) -> u64 {
                *self as u64
            }

            fn from_id(id: u64) -> Result<Self, String> {
                match id {
                    $(x if x == Self::$inst as u64 => Ok(Self::$inst),)+
                    _ => Err(format!("0x{:X} is not a valid instruction ID", id)),
                }
            }
        }
    };
}

/// Macro to extend an instruction set
///
/// For an example, go to this [file](https://github.com/SaadiSave/cambridge-asm/blob/main/cambridge-asm/tests/int_test.rs)
///
/// Due to language limitations, do not use this macro within the same file twice
#[macro_export]
macro_rules! extend {
    ($(#[$outer:meta])* $vis:vis $name:ident extends $parent:ident { $( $inst:ident => $func:expr,)+ }) => {
        extend! { $(#[$outer])* $vis $name extends $parent use std; { $( $inst => $func,)+ } }
    };
    ($(#[$outer:meta])* $vis:vis $name:ident extends $parent:ident $using:item { $( $inst:ident => $func:expr,)+ }) => {
        $(#[$outer])*
        $vis struct $name {
            __private: extend_priv::Combined<$parent>,
        }

        $(#[$outer])*
        pub(crate) mod extend_priv {
            use $crate::inst::InstSet;
            use super::$parent;
            #[repr(u64)]
            #[derive(Clone, Copy)]
            pub enum $name {
                $($inst,)+
                #[allow(non_camel_case_types)]
                LAST_INST_MARKER,
            }

            impl std::str::FromStr for $name {
                type Err = String;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s.to_uppercase().as_str() {
                        $( stringify!($inst) => Ok(Self::$inst),)+
                        _ => Err(String::new()),
                    }
                }
            }

            impl $name {
                fn id(self) -> u64 {
                    self as u64
                }

                fn as_func_ptr(&self) -> $crate::exec::ExecFunc {
                    $using
                    match self {
                        $(Self::$inst => $func,)+
                        Self::LAST_INST_MARKER => panic!("This should never happen, report this as a bug"),
                    }
                }

                fn from_id(id: u64) -> Result<Self, String> {
                    match id {
                        $(x if x == Self::$inst as u64 => Ok(Self::$inst),)+
                        _ => Err(format!("0x{id:X} is not a valid instruction ID")),
                    }
                }
            }

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $(Self::$inst => f.write_str(stringify!($inst)),)+
                        Self::LAST_INST_MARKER => panic!("This should never happen, report this as a bug"),
                    }
                }
            }

            pub enum Combined<T>
            where
                T: $crate::inst::InstSet,
                <T as std::str::FromStr>::Err: std::fmt::Display,
            {
                Extension($name),
                Parent(T),
            }

            impl Combined<$parent> {
                const LAST_INST_MARKER: u64 = $name::LAST_INST_MARKER as u64;

                pub fn id(&self) -> u64 {
                    match self {
                        Self::Extension(ext) => ext.id(),
                        Self::Parent(parent) => Self::LAST_INST_MARKER + parent.id()
                    }
                }

                pub fn from_id(id: u64) -> Result<Self, String> {
                    if id >= $name::LAST_INST_MARKER as u64 {
                        Ok(Combined::Parent($parent::from_id(id - Self::LAST_INST_MARKER)?))
                    } else {
                        Ok(Combined::Extension($name::from_id(id)?))
                    }
                }

                pub fn as_func_ptr(&self) -> $crate::exec::ExecFunc {
                    match self {
                        Self::Extension(e) => e.as_func_ptr(),
                        Self::Parent(p) => p.as_func_ptr(),
                    }
                }
            }

            impl std::str::FromStr for Combined<$parent> {
                type Err = String;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    if let Ok(res) = s.parse::<$name>() {
                        Ok(Combined::Extension(res))
                    } else if let Ok(res) = s.parse::<$parent>() {
                        Ok(Combined::Parent(res))
                    } else {
                        Err(format!("{s} is not an instruction"))
                    }
                }
            }

            impl std::fmt::Display for Combined<$parent> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        Self::Extension(e) => write!(f, "{e}"),
                        Self::Parent(p) => write!(f, "{p}"),
                    }
                }
            }
        }

        $(#[$outer])*
        impl std::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok($name { __private: s.to_uppercase().as_str().parse::<extend_priv::Combined<_>>()? })
            }
        }

        $(#[$outer])*
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.__private)
            }
        }

        $(#[$outer])*
        impl $crate::inst::InstSet for $name {
            fn as_func_ptr(&self) -> $crate::exec::ExecFunc {
                self.__private.as_func_ptr()
            }

            fn id(&self) -> u64 {
                self.__private.id()
            }

            fn from_id(id: u64) -> Result<Self, String> {
                Ok( Self { __private: extend_priv::Combined::from_id(id)? })
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
    pub id: u64,
    pub inst: T,
    pub op: Op,
}

impl<T> Inst<T>
where
    T: InstSet,
    <T as FromStr>::Err: Display,
{
    pub fn new(inst: T, op: Op) -> Self {
        Self {
            id: inst.id(),
            op,
            inst,
        }
    }

    pub fn to_exec_inst(self) -> ExecInst {
        ExecInst::new(self.id, self.inst.as_func_ptr(), self.op)
    }
}
