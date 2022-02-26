// Loop start ("Loop" in Korean, so Unicode works 🎊!)
// Division
환상선: INC RES
    ADD 203,201
    CMP 203,204
    JPN 환상선                 // Jump to loop start if not equal
    DBG RES                 // Show result
    ADD ACC,#b111,#o72
    OUT                     // Print A
    OUT #xa                 // Newline
    END

200 0
201 15  // Divisor
RES:    // Result
203
204 75  // Dividend
