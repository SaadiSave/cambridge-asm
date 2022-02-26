use super::{Context, PasmResult};
use crate::exec::PasmError;
use std::ops::Deref;

#[derive(PartialEq, Debug, Clone)]
pub enum Op {
    Fail(String),
    Acc,
    Ix,
    Cmp,
    Ar,
    Loc(usize),
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
        self.is_register() || matches!(self, Op::Loc(_))
    }

    pub fn is_usizeable(&self) -> bool {
        self.is_read_write() || matches!(self, Op::Literal(_))
    }

    /// will panic if [`Op::is_usizeable`] is not checked first
    pub fn get_val(&self, ctx: &Context) -> Result<usize, PasmError> {
        match self {
            &Op::Literal(val) => Ok(val),
            Op::Loc(loc) => ctx.mem.get(loc),
            reg if reg.is_register() => Ok(ctx.get_register(reg)),
            _ => unreachable!(),
        }
    }
}

impl ToString for Op {
    fn to_string(&self) -> String {
        use Op::*;
        match self {
            Null => "None".to_string(),
            Acc => "ACC".to_string(),
            Ix => "IX".to_string(),
            Cmp => "CMP".to_string(),
            Ar => "ar".to_string(),
            Loc(x) | Literal(x) => format!("{x}"),
            Fail(x) => format!("`{x}` was not parsed successfully"),
            Gpr(x) => format!("r{x}"),
            MultiOp(v) => {
                let mut o = "[".to_string();
                v.iter()
                    .map(Self::to_string)
                    .enumerate()
                    .for_each(|(idx, op)| {
                        if idx == v.len() - 1 {
                            o.push_str(&format!("{op}]"));
                        } else {
                            o.push_str(&format!("{op}, "));
                        }
                    });
                o
            }
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
            '0'..='9' => op.parse::<usize>().unwrap(),
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
            use Op::*;

            if inp.is_empty() {
                Null
            } else if let Ok(x) = inp.parse() {
                Loc(x)
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

pub type OpFun = fn(&mut Context, &Op) -> PasmResult;

#[derive(Clone)]
pub struct Inst {
    pub opfun: OpFun,
    pub op: Op,
}

impl Inst {
    pub fn new(inst: OpFun, op: Op) -> Inst {
        Inst { opfun: inst, op }
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
        pub fn $name($ctx: &mut $crate::exec::Context, $op: & $crate::exec::Op) -> $crate::exec::PasmResult {
            use $crate::exec::Op::*;
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident |$ctx:ident| { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, _: & $crate::exec::Op) -> $crate::exec::PasmResult {
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name(_: &mut $crate::exec::Context, _: & $crate::exec::Op) -> $crate::exec::PasmResult {
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident |$ctx:ident, $op:ident| override { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, $op: & $crate::exec::Op) -> $crate::exec::PasmResult {
            use $crate::exec::Op::*;
            $ctx.override_flow_control();
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident |$ctx:ident| override { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name($ctx: &mut $crate::exec::Context, _: & $crate::exec::Op) -> $crate::exec::PasmResult {
            $ctx.override_flow_control();
            $( $code )*
            Ok(())
        }
    };
    ($(#[$outer:meta])* $name:ident override { $( $code:tt )* }) => {
        $(#[$outer])*
        pub fn $name(ctx: &mut $crate::exec::Context, _: & $crate::exec::Op) -> $crate::exec::PasmResult {
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
