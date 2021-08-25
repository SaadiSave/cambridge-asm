#[macro_use]
extern crate cambridge_asm;

inst! {
    custom | ctx | override { println!("This is a custom instruction"); ctx.mar += 1; }
}

extension! {
    get_instruction extends cambridge_asm::parse::get_fn; {
        "CUSTOM" => custom,
    }
}

#[test]
fn test_ext() {
    const PROG: &str = r#"LDM #65
OUT
CUSTOM
LDM #0
LDM #1
LDM #14
INC ACC
DEC ACC
LDD DIE
ADD #19
OUT
END

NONE:"#;
    cambridge_asm::parse::parse(PROG, get_instruction).exec();
}