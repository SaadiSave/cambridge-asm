    LDD 200
    STO RES
    STO 203
LOOP: LDD RES // Loop start
    INC ACC
    STO RES
    LDD 203
    ADD 201
    STO 203
    CMP 204
    JPN LOOP // Jump to loop start if not equal
    LDD RES
    DBG ACC
    LDM #x3a // Load 58
    ADD #b111 // Add 7
    OUT // Output A
    END

200 0
201 15 // Divisor
RES: // Result
203
204 75 // Dividend
