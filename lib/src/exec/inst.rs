// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    exec::{Context, RtResult},
    inst::Op,
};

/// Function pointer of an instruction called with [`Context`] and [`Op`] at runtime
pub type ExecFunc = fn(&mut Context, &Op) -> RtResult;

/// Runtime representation of an instruction
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
///
/// // Context only
/// inst!(name3 (ctx) { /* Do something with ctx */ });
///
/// // Context and op
/// inst!(name5 (ctx, op) { /* Do something with ctx and op */ });
/// ```
///
/// For further reference, look at the source of the module [`super::io`]
#[macro_export]
macro_rules! inst {
    ($(#[$outer:meta])* $vis:vis $name:ident ($ctx:ident, $op:ident) { $( $code:tt )* }) => {
        $(#[$outer])*
        $vis fn $name($ctx: &mut $crate::exec::Context, $op: & $crate::inst::Op) -> $crate::exec::RtResult {
            use $crate::inst::Op::*;
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $vis:vis $name:ident ($ctx:ident) { $( $code:tt )* }) => {
        $(#[$outer])*
        $vis fn $name($ctx: &mut $crate::exec::Context, _: & $crate::inst::Op) -> $crate::exec::RtResult {
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $vis:vis $name:ident { $( $code:tt )* }) => {
        $(#[$outer])*
        $vis fn $name(_: &mut $crate::exec::Context, _: & $crate::inst::Op) -> $crate::exec::RtResult {
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
            ("200", Addr(200)),
            ("#x80", Literal(128)),
            ("#b001", Literal(1)),
            ("#800", Literal(800)),
            (
                "200,#8,be",
                MultiOp(vec![Addr(200), Literal(8), Fail("be".into())]),
            ),
            ("", Null),
            ("ACC,r10,#x10", MultiOp(vec![Acc, Gpr(10), Literal(16)])),
        ];

        for (op, res) in ops {
            assert_eq!(Op::from(op), res);
        }
    }
}
