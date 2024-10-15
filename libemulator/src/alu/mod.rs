use flags::ALUFlags;
use libstrmisa::Word;

pub mod flags;

pub struct ALU {
    pub flags: ALUFlags,
}

impl ALU {
    pub fn new() -> Self {
        Self {
            flags: ALUFlags::empty(),
        }
    }

    pub fn add(&mut self, a: Word, b: Word) -> Word {
        let value = a.wrapping_add(b);
        let carry = (a as usize + b as usize) > Word::MAX as usize;

        self.flags_by(value, carry)
    }

    pub fn sub(&mut self, a: Word, b: Word) -> Word {
        let value = a.wrapping_sub(b);
        let carry = b > a;

        self.flags_by(value, carry)
    }

    pub fn and(&mut self, a: Word, b: Word) -> Word {
        let value = a & b;
        self.flags_by(value, false)
    }

    pub fn addc(&mut self, a: Word, b: Word) -> Word {
        let carry = if self.flags.contains(ALUFlags::CARRY) {
            1
        } else {
            0
        };
        self.add(a, b + carry)
    }

    pub fn subc(&mut self, a: Word, b: Word) -> Word {
        let carry = if self.flags.contains(ALUFlags::CARRY) {
            1
        } else {
            0
        };
        self.sub(a, b + carry)
    }

    fn flags_by(&mut self, value: Word, carry: bool) -> Word {
        self.flags = if carry {
            ALUFlags::CARRY
        } else {
            ALUFlags::empty()
        } | if value == 0 {
            ALUFlags::ZERO
        } else {
            ALUFlags::empty()
        };

        value
    }
}
