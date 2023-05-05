// This program shows off all the features
mov r0, ptr     // multiple operands
ldm r1, #5      // literals
call print      // call function
end

// procedure to print from linear memory
// inputs:
//  - r0: pointer
//  - r1: length
// returns nothing
print: nop              // no-op
       add r2, r0, r1   // `add` takes the format `destination, operand, operand`
loop:  out (r0)         // indirect addressing syntax; uses the value of the address in r0
       inc r0
       cmp r0, r2
       jpn loop
       zero r0, r1, r2  // zero all operands
       out #xA          // newline; hex literals (octal and binary also available)
       ret              // return to call point


ptr: 201
201 72 // H
202 69 // E
203 76 // L
204 76 // L
205 79 // O
