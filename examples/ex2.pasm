LOOP: LDD ANSWER
ADD NUMONE
STO ANSWER
LDD COUNT
INC ACC
STO COUNT
CMP NUMTWO
JPN LOOP
LDD ANSWER
DBG ACC
END

NUMONE: 625
NUMTWO: 25
COUNT: 0
ANSWER: 0
