use super::{Context, PasmResult};
use crate::parse::get_literal;
use std::ops::Deref;

#[derive(PartialEq, Debug, Clone)]
pub enum Op {
    Str(String),
    Acc,
    Ix,
    Cmp,
    Loc(usize),
    Literal(usize),
    MultiOp(Vec<Op>),
    None,
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
        match self {
            Op::None => true,
            _ => false,
        }
    }
}

impl ToString for Op {
    fn to_string(&self) -> String {
        use Op::*;
        match self {
            None => "None".to_string(),
            Acc => "ACC".to_string(),
            Ix => "IX".to_string(),
            Cmp => "CMP".to_string(),
            Loc(x) | Literal(x) => format!("{}", x),
            Str(x) => x.clone(),
            MultiOp(v) => format!("{:?}", v),
        }
    }
}

impl From<Op> for String {
    fn from(op: Op) -> Self {
        op.to_string()
    }
}

impl<T: Deref<Target = str>> From<T> for Op {
    fn from(inp: T) -> Self {
        fn get_op(inp: &str) -> Op {
            use Op::*;

            if inp.is_empty() {
                None
            } else if let Ok(x) = inp.parse() {
                Loc(x)
            } else if inp.contains('#') {
                Literal(get_literal(inp.into()).parse().unwrap())
            } else {
                match inp.to_lowercase().as_str() {
                    "acc" => Acc,
                    "cmp" => Cmp,
                    "ix" => Ix,
                    _ => Str(inp.into()),
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

pub type Func = fn(&mut Context, Op) -> PasmResult;

pub type Cmd = (Func, Op);

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
        pub fn $name($ctx: &mut $crate::exec::Context, $op: $crate::exec::Op) -> $crate::exec::PasmResult {
            use $crate::exec::Op;
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident |$ctx:ident| { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, _: $crate::exec::Op) -> $crate::exec::PasmResult {
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name(_: &mut $crate::exec::Context, _: $crate::exec::Op) -> $crate::exec::PasmResult {
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident |$ctx:ident, $op:ident| override { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, $op: $crate::exec::Op) -> $crate::exec::PasmResult {
            use $crate::exec::Op;
            $ctx.override_flow_control();
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident |$ctx:ident| override { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, _: $crate::exec::Op) -> $crate::exec::PasmResult {
            $ctx.override_flow_control();
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident override { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name(ctx: &mut $crate::exec::Context, _: $crate::exec::Op) -> $crate::exec::PasmResult {
            ctx.override_flow_control();
            $( $code )*
            Ok(())
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use Op::*;

    #[test]
    fn op_parsing() {
        let ops = [
            ("200", Loc(200)),
            ("#x80", Literal(128)),
            ("#b001", Literal(1)),
            ("#800", Literal(800)),
            (
                "200,#8,be",
                MultiOp(vec![Loc(200), Literal(8), Str("be".into())]),
            ),
            ("", None),
            ("ACC,100,#x10", MultiOp(vec![Acc, Loc(100), Literal(16)])),
        ];

        for (op, res) in ops {
            assert_eq!(Op::from(op), res);
        }
    }
}
