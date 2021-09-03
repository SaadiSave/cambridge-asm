// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{exec::PasmError, inst};
use std::fmt::Debug;
use std::io::Read;

inst!(end {});

inst!(
    out | ctx | {
        let x = ctx.acc;

        if x > 255 {
            return Err(PasmError::from(format!(
                "The value in the ACC, `{}`, is not a valid UTF-8 byte.",
                x
            )));
        }

        let out = x as u8 as char;

        print!("{}", out);
    }
);

inst!(
    inp | ctx | {
        let mut x = [0; 1];

        std::io::stdin()
            .read_exact(&mut x)
            .expect("Unable to read stdin");

        ctx.acc = x[0] as usize;
    }
);

// Custom instruction for debug logging
inst!(
    #[cfg(not(feature = "cambridge"))]
    dbg | ctx,
    op | {
        let out: Box<dyn Debug> = match op {
            Op::Loc(x) => Box::new(ctx.mem.get(&x)?),
            Op::Ix => Box::new(ctx.ix),
            Op::Acc => Box::new(ctx.acc),
            Op::Cmp => Box::new(ctx.cmp),
            Op::None => Box::new(ctx),
            _ => return Err(PasmError::InvalidOperand),
        };

        println!("{:?}", out);
    }
);

// Raw input - directly input integers
inst!(
    #[cfg(not(feature = "cambridge"))]
    rin | ctx
        | {
            let mut x = String::new();

            std::io::stdin()
                .read_line(&mut x)
                .expect("Unable to read stdin");

            x.ends_with('\n').then(|| x.pop());
            x.ends_with('\r').then(|| x.pop());

            ctx.acc = x
                .parse()
                .unwrap_or_else(|_| panic!("'{}' is not an integer", x));
        }
);
