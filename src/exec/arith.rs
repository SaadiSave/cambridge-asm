use super::{Context, Op};

pub fn add(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc += ctx.mem.get(&x);

    ctx.increment();
}

pub fn addm(ctx: &mut Context, op: Op) {
    let x: usize = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc += x;

    ctx.increment();
}

pub fn sub(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc -= ctx.mem.get(&x);

    ctx.increment();
}

pub fn subm(ctx: &mut Context, op: Op) {
    let x: usize = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc -= x;

    ctx.increment();
}

pub fn inc(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand");

    match x.as_str() {
        "ix" | "IX" => ctx.ix += 1,
        "acc" | "ACC" => ctx.acc += 1,
        _ => panic!("{} is an invalid register", &x)
    }

    ctx.increment();
}

pub fn dec(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand");

    match x.as_str() {
        "ix" | "IX" => ctx.ix -= 1,
        "acc" | "ACC" => ctx.acc -= 1,
        _ => panic!("{} is an invalid register", &x)
    }

    ctx.increment();
}
