pub const REG_COUNT: usize = 2usize.pow(4);
pub const MEM_START: usize = 1024;
pub const MEM_LENGTH: usize = 2usize.pow(16) - MEM_START;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    LoadI { dest: usize, value: u16 },
    Load { dest: usize, src_addr: usize },
    Store { dest_addr: usize, src: usize },
    Cpy { dest: usize, src: usize },
    Jmp { addr: usize },
    JmpC { addr: usize },
    JmpZ { addr: usize },
    Add { a: usize, b: usize },
    Sub { a: usize, b: usize },
    And { a: usize, b: usize },
    Halt,
}

impl Instruction {
    pub fn build(self) -> Vec<u8> {
        let (opcode, a, b, imm) = match self {
            Self::LoadI { dest, value } => (1, dest, 0, Some(value)),
            Self::Load { dest, src_addr } => (2, dest, src_addr, None),
            Self::Store { dest_addr, src } => (3, dest_addr, src, None),
            Self::Cpy { dest, src } => (4, dest, src, None),
            Self::Jmp { addr } => (5, addr, 0, None),
            Self::JmpC { addr } => (6, addr, 0, None),
            Self::JmpZ { addr } => (7, addr, 0, None),
            Self::Add { a, b } => (8, a, b, None),
            Self::Sub { a, b } => (9, a, b, None),
            Self::And { a, b } => (15, a, b, None),
            Self::Halt => (24, 0, 0, None),
        };

        let instruction_word = ((opcode << 10) | (a << 6) | (b << 2)) as u16;

        let mut bytes = Vec::from(Self::word_to_bytes(instruction_word));

        if let Some(immediate_word) = imm {
            bytes.extend(Self::word_to_bytes(immediate_word));
        }

        bytes
    }

    fn word_to_bytes(word: u16) -> [u8; 2] {
        [((word & 0xFF00) >> 8) as u8, (word & 0x00FF) as u8]
    }
}
