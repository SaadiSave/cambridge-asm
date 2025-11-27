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
        sqrtf (ctx, op) {
            match op {
                op if op.is_usizeable() => {
                    #[cfg(target_pointer_width = "32")]
                    let float = f32::from_bits(ctx.read(op)? as _);

                    #[cfg(target_pointer_width = "64")]
                    let float = f64::from_bits(ctx.read(op)? as _);

                    ctx.acc = float.sqrt().to_bits() as usize;
                },
                _ => return Err(RtError::InvalidOperand)
            }
        }
    }

    inst! {
        outf (ctx, op) {
            match op {
                op if op.is_usizeable() => {
                    #[cfg(target_pointer_width = "32")]
                    let float = f32::from_bits(ctx.read(op)? as _);

                    #[cfg(target_pointer_width = "64")]
                    let float = f64::from_bits(ctx.read(op)? as _);

                    writeln!(ctx.io.write, "{float}")?;
                }
                _ => return Err(RtError::InvalidOperand)
            }
        }
    }

    inst_set! {
        Custom {
            SQRTF => sqrtf,
            OUTF => outf,
            END => cambridge_asm::exec::io::end,
        }
    }

    #[test]
    fn custom() {
        #[cfg(target_pointer_width = "64")]
        const PROG: &str = r#"sqrtf #x4000000000000000
outf acc
end

none:
"#;

        #[cfg(target_pointer_width = "32")]
        const PROG: &str = r#"sqrtf #0x40000000
outf acc
end

none:
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

        #[cfg(target_pointer_width = "32")]
        assert!(f32::from_bits(e.ctx.acc as _) >= 1.4);

        #[cfg(target_pointer_width = "64")]
        assert!(f64::from_bits(e.ctx.acc as _) >= 1.4);

        println!("{}", String::from_utf8_lossy(out.to_vec().as_slice()));
    }
}
