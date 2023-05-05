# cambridge-asm

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/saadisave/cambridge-asm/Test?style=for-the-badge)](https://github.com/SaadiSave/cambridge-asm/actions/workflows/test.yml) [![Crates.io](https://img.shields.io/crates/v/cambridge-asm?style=for-the-badge)](https://crates.io/crates/cambridge-asm)

## Extending the instruction set

### With macros

```rust
#[macro_use]
extern crate cambridge_asm;

// Import `Core` instruction set
use cambridge_asm::parse::Core;

// Custom instruction
inst! {
    ext (ctx) {
        // I/O accessed with ctx.io
        // Output is ctx.io.write
        // Use with write! or writeln! macros
        writeln!(ctx.io.write, "This is a custom instruction").unwrap();

        // Set r0 to detect custom instruction call
        ctx.gprs[0] = 20;
    }
}

// Extend `Core` instruction set
extend! {
    Ext extends Core {
        EXT => ext,
    }
}
```

### Without macros

```rust
// Import `Core` instruction set
use cambridge_asm::parse::Core;

// Imports of essential types
use cambridge_asm::{
    exec::{Context, ExecFunc, PasmResult},
    inst::{InstSet, Op},
};

pub fn ext(ctx: &mut Context, _: &Op) -> PasmResult {
    // I/O accessed with ctx.io
    // Output is ctx.io.write
    // Use with write! or writeln! macros
    writeln!(ctx.io.write, "This is a custom instruction").unwrap();

    // Set r0 to detect custom instruction call
    ctx.gprs[0] = 20;

    // Return Ok
    Ok(())
}

enum Ext {
    EXT,
    Parent(Core),
}

impl std::str::FromStr for Ext {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "EXT" => Ok(Self::EXT),
            s => Ok(Self::Parent(s.parse::<Core>()?)),
        }
    }
}

impl std::fmt::Display for Ext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EXT => f.write_str("EXT"),
            Self::Parent(p) => write!(f, "{}", p),
        }
    }
}

impl InstSet for Ext {
    fn as_func_ptr(&self) -> ExecFunc {
        match self {
            Self::EXT => ext,
            Self::Parent(p) => p.as_func_ptr(),
        }
    }

    fn from_func_ptr(f: ExecFunc) -> Result<Self, String> {
        const EXT: ExecFunc = ext;

        match f {
            EXT => Ok(Self::EXT),
            f => Ok(Self::Parent(Core::from_func_ptr(f)?)),
        }
    }
}
```

### Usage

```rust
fn main() {
    use cambridge_asm::{exec::Io, parse::jit};

    const PROG: &str = r#"EXT
END

NONE:
"#;
    let out = TestStdout::new(vec![]);

    let mut e = jit::<Ext>(PROG, Io::default()).unwrap();
    e.exec::<Ext>();

    // Check if r0 == 20
    assert_eq!(e.ctx.gprs[0], 20);
}
```

Refer to [src/parse.rs](./src/parse.rs) to see how the `Extended` instruction set is made using the macro

## Using a completely custom instruction set

```rust
#[macro_use]
extern crate cambridge_asm;

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

// Define a new instruction set
inst_set! {
    Custom {
        H => h,
        E => e,
        L => l,
        O => o,
        END => cambridge_asm::exec::io::end,
    }
}

// Using it
fn main() {
    use cambridge_asm::{exec::Io, parse::jit};

    // Outputs "HELLO"
    const PROG: &str = r#"H
E
L
L
O
END

NONE:
"#;

    let out = TestStdout::new(vec![]);

    let mut e = jit::<Custom>(PROG, Io::default()).unwrap();
    e.exec::<Custom>();
}
```
