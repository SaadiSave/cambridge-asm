use super::{Context, Op};

pub fn ldm(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc = x;

    ctx.increment();
}

pub fn ldd(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc = ctx.mem.get(&x);

    ctx.increment();
}

pub fn ldi(ctx: &mut Context, op: Op) {
    let mut x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    x = ctx.mem.get(&x);

    ctx.acc = ctx.mem.get(&x);

    ctx.increment();
}

pub fn ldx(ctx: &mut Context, op: Op) {
    let mut x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");
    x += ctx.ix;

    ctx.acc = ctx.mem.get(&x);

    ctx.increment();
}

pub fn ldr(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.ix = x;

    ctx.increment();
}

pub fn mov(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand");

    match x.as_str() {
        "ix" | "IX" => ctx.ix = ctx.acc,
        _ => panic!("{} is an invalid register", &x)
    }

    ctx.increment();
}

pub fn sto(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.mem.write(&x, ctx.acc);

    ctx.increment();
}
