#[macro_use]
extern crate cambridge_asm;

include!("../test_stdio.rs");

/// Extending the `Core` instruction set
mod extension {
    use super::TestStdio;
    use cambridge_asm::parse::Core;
    use std::io::Write;

    inst! {
        ext (ctx) {
            writeln!(ctx.io.write, "This is a custom instruction").unwrap();
            ctx.gprs[0] = 20;
        }
    }

    extend! {
        Ext extends Core use super::*; {
            EXT => ext,
        }
    }

    #[test]
    fn extension() {
        const PROG: &str = r#"LDM #65
OUT
EXT
LDM #0
LDM #1
LDM #14
INC ACC
DEC ACC
LDD NONE
ADD #65
OUT
END

NONE:
"#;
        let out = TestStdio::new(vec![]);

        let mut e = cambridge_asm::parse::jit::<Ext>(PROG, make_io!(std::io::stdin(), out.clone()))
            .unwrap();
        e.exec::<Ext>();
        assert_eq!(e.ctx.acc, 65);
        assert_eq!(e.ctx.gprs[0], 20);
        assert_eq!(out.to_vec(), b"AThis is a custom instruction\nA");
    }
}

/// Using a completely custom instruction set
mod custom {
    use super::TestStdio;
    use cambridge_asm::exec::RtError;
    use std::io::Write;

    inst! {
        from (ctx, op) {
            match op {
                Fail(from) => writeln!(ctx.io.write, "From {from}")?,
                Null => writeln!(ctx.io.write, "From Pseudoassembly")?,
                _ => return Err(RtError::InvalidOperand)
            }
        }
    }

    inst! {
        greet (ctx, op) {
            match op {
                Fail(msg) => writeln!(ctx.io.write, "Hello, {msg}!")?,
                Null => writeln!(ctx.io.write, "Hello!")?,
                _ => return Err(RtError::InvalidOperand)
            }
        }
    }

    inst_set! {
        Custom {
            GREET => greet,
            FROM => from,
            END => cambridge_asm::exec::io::end,
        }
    }

    #[test]
    fn custom() {
        const PROG: &str = r#"GREET
FROM Pseudoassembly
END

NONE:
"#;

        let out = TestStdio::new(vec![]);

        let mut e =
            cambridge_asm::parse::jit::<Custom>(PROG, make_io!(std::io::stdin(), out.clone()))
                .unwrap_or_else(|e| {
                    e.iter()
                        .for_each(|(r, e)| println!("{} : {e:?}", &PROG[r.clone()]));
                    panic!()
                });
        e.exec::<Custom>();

        assert_eq!(out.to_vec(), b"Hello!\nFrom Pseudoassembly\n");
    }
}
