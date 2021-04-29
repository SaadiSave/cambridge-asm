use super::{Context, Op};

pub fn and(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc &= ctx.mem.get(&x);
}

pub fn andm(ctx: &mut Context, op: Op) {
    let x: usize = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc &= x;
}

pub fn or(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc |= ctx.mem.get(&x);
}

pub fn orm(ctx: &mut Context, op: Op) {
    let x: usize = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc |= x;
}

pub fn xor(ctx: &mut Context, op: Op) {
    let x = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc ^= ctx.mem.get(&x);
}

pub fn xorm(ctx: &mut Context, op: Op) {
    let x: usize = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc ^= x;
}

pub fn lsl(ctx: &mut Context, op: Op) {
    let x: usize = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc <<= x;
}

pub fn lsr(ctx: &mut Context, op: Op) {
    let x: usize = op
        .expect("No operand")
        .parse()
        .expect("Operand is not an integer");

    ctx.acc >>= x;
}
