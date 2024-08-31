#include "rules.asm"

#bank code
loadi %0, $loop
jmp %0

loadi %1, $0xDEAD ; Unreachable

loop:
    jmp %0
    loadi %2, $0xDEAD ; Unreachable
