// This is a demonstration of indirect addressing
// This program prints HELLO
LOOP: LDI PTR
    OUT
    LDD PTR
    INC ACC
    STO PTR
    CMP #207
    JPN LOOP
    END


PTR: 201
201 72 // H
202 69 // E
203 76 // L
204 76 // L
205 79 // O
206 10 // \n
