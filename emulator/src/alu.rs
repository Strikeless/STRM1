use super::{flags::Flags, Word};

pub struct ALU {
    pub flags: Flags,
}

impl ALU {
    pub fn new() -> Self {
        Self {
            flags: Flags::empty(),
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
        let carry = if self.flags.contains(Flags::CARRY) {
            1
        } else {
            0
        };
        self.add(a, b + carry)
    }

    pub fn subc(&mut self, a: Word, b: Word) -> Word {
        let carry = if self.flags.contains(Flags::CARRY) {
            1
        } else {
            0
        };
        self.sub(a, b + carry)
    }

    fn flags_by(&mut self, value: Word, carry: bool) -> Word {
        self.flags = if carry { Flags::CARRY } else { Flags::empty() }
            | if value == 0 {
                Flags::ZERO
            } else {
                Flags::empty()
            };

        value
    }
}
