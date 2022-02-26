// This program demonstrates the use of functions
ldm r1,#13  // Set r1 to 13
ldm r2,#5   // Set r2 to 5
call mul    // Call function
out r0      // Outputs 'A'
mov acc,r0  // Copy r0 to acc for unit test
end         // end is important here, because address space continues below

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
