// Copyright (c) 2021 Saadi Save
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Context, Op};

pub fn jmp(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.mar = x;
}

pub fn cmp(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.cmpr = ctx.acc == ctx.mem.get(&x);

    ctx.increment()
}

pub fn cmpm(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.cmpr = ctx.acc == x;

    ctx.increment()
}

pub fn cmi(ctx: &mut Context, op: Op) {
    let mut x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    x = ctx.mem.get(&x);

    ctx.cmpr = ctx.acc == ctx.mem.get(&x);

    ctx.increment();
}

pub fn jpe(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    if ctx.cmpr {
        ctx.mar = x;
    } else {
        ctx.increment();
    }
}

pub fn jpn(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    if ctx.cmpr {
        ctx.increment();
    } else {
        ctx.mar = x;
    }
}
