LOOP: LDX 201
OUT
INC IX
LDD CNT
INC ACC
STO CNT
CMP #5
JPN LOOP
LDM #10 // Code for newline
OUT // Output newline
END // This program prints HELLO


CNT: 0
201 72 // H
202 69 // E
203 76 // L
204 76 // L
205 79 // O