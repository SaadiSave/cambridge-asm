#[macro_use]
extern crate cambridge_asm;

use std::{io::Write, rc::Rc, sync::RwLock};

#[derive(Clone)]
#[repr(transparent)]
pub struct TestStdout(Rc<RwLock<Vec<u8>>>);

impl TestStdout {
    pub fn new(s: impl std::ops::Deref<Target = [u8]>) -> Self {
        Self(Rc::new(RwLock::new(s.to_vec())))
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.0.read().unwrap().clone()
    }
}

impl Write for TestStdout {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write().unwrap().extend(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

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

        let mut e =
            cambridge_asm::parse::jit::<Ext, _>(PROG, make_io!(std::io::stdin(), out.clone()));
        e.exec::<Ext>();
        assert_eq!(e.ctx.acc, 65);
        assert_eq!(e.ctx.gprs[0], 20);
        assert_eq!(out.to_vec(), b"AThis is a custom instruction\nA");
    }
}

/// Using a completely custom instruction set
mod custom {
    use super::TestStdout;
    inst! {
        h (ctx) {
            write!(ctx.io.write, "H").unwrap();
        }
    }

    inst! {
        e (ctx) {
            write!(ctx.io.write, "E").unwrap();
        }
    }

    inst! {
        l (ctx) {
            write!(ctx.io.write, "L").unwrap();
        }
    }

    inst! {
        o (ctx) {
            write!(ctx.io.write, "O").unwrap();
        }
    }

    inst_set! {
        Custom {
            H => h,
            E => e,
            L => l,
            O => o,
            END => cambridge_asm::exec::io::end,
        }
    }

    #[test]
    fn custom() {
        const PROG: &str = r#"H
E
L
L
O
END

NONE:
"#;

        let out = TestStdout::new(vec![]);

        let mut e =
            cambridge_asm::parse::jit::<Custom, _>(PROG, make_io!(std::io::stdin(), out.clone()));
        e.exec::<Custom>();

        assert_eq!(out.to_vec(), b"HELLO");
    }
}
