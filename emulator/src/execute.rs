use super::{flags::Flags, Emulator, Word};

impl Emulator {
    pub fn execute_to_halt(&mut self) {
        while !self.halted {
            self.execute_instruction();
        }
    }

    pub fn execute_instruction(&mut self) {
        let instruction = self.pc_next();

        let [opcode, ra, rb] = [
            instruction >> (Word::BITS - 6) & 0b111111,
            instruction >> (Word::BITS - 10) & 0b11,
            instruction >> (Word::BITS - 14) & 0b11,
        ];

        match opcode {
            // nop
            0 => {}

            // loadi
            1 => self.w(ra, |this, ra| {
                *ra = this.pc_next()
            }),

            // load
            2 => self.wr(ra, rb, |this, dest, src_addr| {
                *dest = *this.memory(*src_addr);
            }),

            // store
            3 => self.rr(ra, rb, |this, dest_addr, src| {
                *this.memory_mut(*dest_addr) = *src;
            }),

            // cpy
            4 => self.wr(ra, rb, |_, dest, src| {
                *dest = *src;
            }),

            // jmp
            5 => self.r(ra, |this, addr| {
                this.program_counter = *addr;
            }),

            // jmpc
            6 => self.r(ra, |this, addr| {
                if this.alu.flags.contains(Flags::CARRY) {
                    this.program_counter = *addr;
                }
            }),

            // jmpz
            7 => self.r(ra, |this, addr| {
                if this.alu.flags.contains(Flags::ZERO) {
                    this.program_counter = *addr;
                }
            }),

            // add
            8 => self.wr(ra, rb, |this, a, b| {
                *a = this.alu.add(*a, *b);
            }),

            // sub
            9 => self.wr(ra, rb, |this, a, b| {
                *a = this.alu.sub(*a, *b);
            }),

            // addc
            12 => self.wr(ra, rb, |this, a, b| {
                *a = this.alu.addc(*a, *b);
            }),

            // subc
            13 => self.wr(ra, rb, |this, a, b| {
                *a = this.alu.subc(*a, *b);
            }),

            // and
            15 => self.wr(ra, rb, |this, a, b| {
                *a = this.alu.and(*a, *b);
            }),

            // loadh
            20 => self.wr(ra, rb, |this, dest, src_addr| {
                let value = *this.memory(*src_addr);
                *dest = (*dest & 0x00FF) | (value & 0xFF00);
            }),

            // loadl
            21 => self.wr(ra, rb, |this, dest, src_addr| {
                let src = *this.memory(*src_addr);
                *dest = (*dest & 0xFF00) | ((src & 0xFF00) >> 8);
            }),

            // storeh
            22 => self.rr(ra, rb, |this, dest_addr, src| {
                let dest = this.memory_mut(*dest_addr);
                *dest = (*dest & 0x00FF) | (src & 0xFF00);
            }),

            // storel
            23 => self.rr(ra, rb, |this, dest_addr, src| {
                let dest = this.memory_mut(*dest_addr);
                *dest = (*dest & 0x00FF) | ((src & 0x00FF) << 8);
            }),

            // halt
            24 => self.halted = true,

            _ => {
                eprintln!("Illegal opcode {:b}", opcode);
                // Might wanna trigger an interrupt or something once those are a thing...
            }
        }
    }

    fn r<F>(&mut self, ra: Word, mut exec: F)
    where
        F: FnMut(&mut Self, &Word),
    {
        let a = *self.register(ra);
        exec(self, &a);
    }
    
    fn w<F>(&mut self, ra: Word, mut exec: F)
    where
        F: FnMut(&mut Self, &mut Word),
    {
        let mut a = *self.register(ra);

        exec(self, &mut a);
        *self.register_mut(ra) = a;
    }

    fn rr<F>(&mut self, ra: Word, rb: Word, mut exec: F)
    where
        F: FnMut(&mut Self, &Word, &Word),
    {
        let a = *self.register(ra);
        let b = *self.register(rb);
        exec(self, &a, &b);
    }

    fn wr<F>(&mut self, ra: Word, rb: Word, mut exec: F)
    where
        F: FnMut(&mut Self, &mut Word, &Word),
    {
        let mut a = *self.register(ra);
        let b = *self.register(rb);
        
        exec(self, &mut a, &b);
        *self.register_mut(ra) = a;
    }

    fn pc_next(&mut self) -> Word {
        let data = *self.memory(self.program_counter);

        self.program_counter = self
            .program_counter
            .wrapping_add(super::AUS_PER_WORD as Word);

        data
    }
}
