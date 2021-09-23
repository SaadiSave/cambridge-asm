    MOV 200,RES // Multiple operators are supported too !
환상선: LDD RES // Loop start ("Loop" in Korean [Unicode works 🎊!])
    INC ACC
    STO RES
    LDD 203
    ADD 201
    STO 203
    CMP 204
    JPN 환상선 // Jump to loop start if not equal
    DBG
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
