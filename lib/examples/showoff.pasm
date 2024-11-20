// This program shows off all the features
ldm r0, #201    // multiple operands
ldm r1, #5      // literals
call print      // call function
ldm r0, #206
ldm r1, #5
call read
mov acc, 206
ldm r0, #206
ldm r1, #5
call print
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
       
// procedure to read input to linear memory
// inputs:
//  - r0: pointer
//  - r1: length
// returns nothing
read:  nop              // no-op
       add r2, r0, r1   // `add` takes the format `destination, operand, operand`
loop2: in (r0)          // indirect addressing syntax; uses the value of the address in r0
       inc r0
       cmp r0, r2
       jpn loop2
       zero r0, r1, r2  // zero all operands
       ret              // return to call point


201 72 // H
202 69 // E
203 76 // L
204 76 // L
205 79 // O
206 [0;5] // linear memory syntax; initialises zeroed memory from 206 to 210
