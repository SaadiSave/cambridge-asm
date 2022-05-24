// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    exec::{Context, PasmResult},
    inst::Op,
};

pub type ExecFunc = fn(&mut Context, &Op) -> PasmResult;

#[derive(Clone)]
pub struct ExecInst {
    pub func: ExecFunc,
    pub op: Op,
}

impl ExecInst {
    pub fn new(inst: ExecFunc, op: Op) -> Self {
        Self { func: inst, op }
    }
}

/// Macro to generate an instruction implementation
///
/// # Examples
/// ```
/// use cambridge_asm::inst;
///
/// // No Context
/// inst!(name1 { /* Do something that doesn't need context or op*/ });
/// // Flow control override
/// inst!(name2 override { /* */ });
///
/// // Context only
/// inst!(name3 | ctx | { /* Do something with ctx */ });
/// // Override
/// inst!(name4 | ctx | override { /* */ });
///
/// // Context and op
/// inst!(name5 | ctx, op | { /* Do something with ctx and op */ });
/// // Override
/// inst!(name6 | ctx, op | override { /* */ });
/// ```
///
/// For further reference, look at the source of the module [`exec::io`]
#[macro_export]
macro_rules! inst {
    ($(#[$outer:meta])* $name:ident |$ctx:ident, $op:ident| { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, $op: & $crate::inst::Op) -> $crate::exec::PasmResult {
            use $crate::inst::Op::*;
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident |$ctx:ident| { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, _: & $crate::inst::Op) -> $crate::exec::PasmResult {
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name(_: &mut $crate::exec::Context, _: & $crate::inst::Op) -> $crate::exec::PasmResult {
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident |$ctx:ident, $op:ident| override { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, $op: & $crate::inst::Op) -> $crate::exec::PasmResult {
            use $crate::inst::Op::*;
            $ctx.override_flow_control();
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident |$ctx:ident| override { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, _: & $crate::inst::Op) -> $crate::exec::PasmResult {
            $ctx.override_flow_control();
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident override { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name(ctx: &mut $crate::exec::Context, _: & $crate::inst::Op) -> $crate::exec::PasmResult {
            ctx.override_flow_control();
            $( $code )*
            Ok(())
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inst::Op::*;

    #[test]
    fn op_parsing() {
        let ops = [
            ("200", Loc(200)),
            ("#x80", Literal(128)),
            ("#b001", Literal(1)),
            ("#800", Literal(800)),
            (
                "200,#8,be",
                MultiOp(vec![Loc(200), Literal(8), Fail("be".into())]),
            ),
            ("", Null),
            ("ACC,r10,#x10", MultiOp(vec![Acc, Gpr(10), Literal(16)])),
        ];

        for (op, res) in ops {
            let x = Op::from(op);
            assert_eq!(x, res);
        }
    }
}
