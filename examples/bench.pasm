// This program is to be used for benchmarking
// Current performance is equivalent to a late 90s PC
ldm r1,#10000       // Set r1 to 13
ldm r2,#100000000   // Set r2 to 5
call mul            // Call function
end                 // end is important here, because address space continues below

// Multiply two numbers
// inputs: r1, r2
// ret: r0
// optimisation: r2 < r1
mul: add r0,r1   // First param to return value
     inc r20     // Increment count
     cmp r20,r2  // Compare count to second value
     jpn mul     // Repeat if not equal
     ldm r10,#0  // Clear working register
     ret


NONE:
