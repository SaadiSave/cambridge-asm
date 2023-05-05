#[macro_use]
extern crate cambridge_asm;

include!("../test_stdout.rs");

/// Extending the `Core` instruction set
mod extension {
    use super::TestStdout;
    use cambridge_asm::parse::Core;

    inst! {
        ext (ctx) {
            writeln!(ctx.io.write, "This is a custom instruction").unwrap();
            ctx.gprs[0] = 20;
        }
    }

    extend! {
        Ext extends Core {
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
        let out = TestStdout::new(vec![]);

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
    use cambridge_asm::exec::PasmError;

    use super::TestStdout;
    inst! {
        from (ctx, op) {
            match op {
                Fail(from) => writeln!(ctx.io.write, "From {from}").unwrap(),
                Null => writeln!(ctx.io.write, "From Pseudoassembly").unwrap(),
                _ => return Err(PasmError::InvalidOperand)
            }
        }
    }

    inst! {
        greet (ctx, op) {
            match op {
                Fail(msg) => writeln!(ctx.io.write, "Hello, {msg}!").unwrap(),
                Null => writeln!(ctx.io.write, "Hello!").unwrap(),
                _ => return Err(PasmError::InvalidOperand)
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

        let out = TestStdout::new(vec![]);

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
