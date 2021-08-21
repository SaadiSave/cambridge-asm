// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{exec::PasmError, inst};
use std::io::Read;

inst!(end {});

inst!(
    out | ctx | {
        let x = ctx.acc;

        if x > 255 {
            return Err(PasmError::from(format!(
                "The value in the ACC, `{}`, is not a valid UTF-8 byte.",
                &x
            )));
        }

        let out = x as u8 as char;

        print!("{}", &out);
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
        let x = op.ok_or(PasmError::NoOperand)?;

        let out = match x.as_str() {
            "ix" | "IX" => ctx.ix,
            "acc" | "ACC" => ctx.acc,
            _ => {
                if let Ok(s) = x.parse() {
                    ctx.mem.get(&s)?
                } else {
                    return Err(PasmError::from(format!(
                        "{} is not a register or a memory address",
                        &x
                    )));
                }
            }
        };

        println!("{}", &out);
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
                .unwrap_or_else(|_| panic!("'{}' is not an integer", &x));
        }
);
