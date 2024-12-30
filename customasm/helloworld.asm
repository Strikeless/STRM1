#include "rules.asm"

#bank code
loadi %0, $msg_data ; Load the message data address to register 0
loadi %1, $msg_var ; Load the message variable address to register 1

char_loop:
    loadl %3, %0 ; Load character from data to %3
    storel %1, %3 ; Store the character to the variable

    and %3, %3 ; And the character with itself to update flags...
    
    ; ...then break out of the char loop if the zero flag is set (character was zero aka null-byte)
    loadi %3, $end
    jmpz %3

    ; Increment the data and variable pointers to point to the next character
    loadi %3, $1
    add %0, %3
    add %1, %3

    ; Jump back to the beginning of the loop
    loadi %3, $char_loop
    jmp %3

end:
    loadi %2, $1337
    halt

msg_data:
    #d "Hello, world!\0"
MSG_LEN = $ - msg_data

#bank ram
msg_var: #res MSG_LEN
