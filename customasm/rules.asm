#once

#bankdef code {
    #outp 0*8
    #addr 0
    #size 1024
    #bits 8
    #labelalign 8 ; Only 8-bit alignment should be required, this is just a workaround for endianness problems in the emulator.
    #fill
}

#bankdef ram {
    #addr       1024
    #addr_end   65536
    #bits       8
}

#fn instr       (opcode)        => instr_rr(opcode, %0, %0)
#fn instr_r     (opcode, ra)     => instr_rr(opcode, ra, %0)
#fn instr_rr    (opcode, ra, rb)  => opcode`6 @ ra`4 @ rb`4 @ 0`2

#subruledef reg {
    %{index: u4} => index
}

#subruledef imm {
    ${value: u16} => value
}

#ruledef {
    nop                                     => instr    (0)                     ; No-operation
    
    loadi {dest: reg}, {value: imm}         => instr_r  (1, dest) @ value       ; %dest = $value
    load {dest: reg}, {src_addr: reg}       => instr_rr (2, dest, src_addr)     ; %dest = MEM[%src_addr]
    store {dest_addr: reg}, {src: reg}      => instr_rr (3, dest_addr, src)     ; MEM[%dest_addr] = %src
    cpy {dest: reg}, {src: reg}             => instr_rr (4, dest, src)          ; %dest = %src

    jmp {addr: reg}                         => instr_r  (5, addr)               ; PC = %addr
    jmpc {addr: reg}                        => instr_r  (6, addr)               ; if carry { PC = %addr }
    jmpz {addr: reg}                        => instr_r  (7, addr)               ; if zero { PC = %addr }

    add {a: reg}, {b: reg}                  => instr_rr (8, a, b)               ; %a = %a + %b
    sub {a: reg}, {b: reg}                  => instr_rr (9, a, b)               ; %a = %a - %b
    ; 10, 11 reserved for mul and div
    addc {a: reg}, {b: reg}                 => instr_rr (12, a, b)              ; %a = %a + (%b + carry)
    subc {a: reg}, {b: reg}                 => instr_rr (13, a, b)              ; %a = %a - (%b + carry)
    ; 14 reserved for mulc

    and {a: reg}, {b: reg}                  => instr_rr (15, a, b)              ; %a = %a & %b
    ; 16, 17, 18, 19, 20, 21 reserved for or, not, xor, nand, shl, shr

    ; Load and store that only operate on the high or low byte of the register and the high byte in memory,
    ; high byte being at the exact address specified and low byte being at the next address (big endian).
    ; The byte in memory is the same in all of these instructions (the high byte), it's the byte in the register that changes.
    loadh {dest: reg}, {src_addr: reg}      => instr_rr (22, dest, src_addr)    ; high %dest = high MEM[%src_addr]
    loadl {dest: reg}, {src_addr: reg}      => instr_rr (23, dest, src_addr)    ; low %dest = high MEM[%src_addr]
    storeh {dest_addr: reg}, {src: reg}     => instr_rr (24, dest_addr, src)    ; high MEM[%dest_addr] = high %src
    storel {dest_addr: reg}, {src: reg}     => instr_rr (25, dest_addr, src)    ; high MEM[%dest_addr] = low %src

    halt                                    => instr    (26)                    ; Stops linear code execution
}
