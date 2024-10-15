use std::fmt::Display;

use bimap::BiMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref KIND_OPCODE_BIMAP: BiMap<InstructionKind, usize> = BiMap::from_iter([
        (InstructionKind::Nop, 0),
        (InstructionKind::LoadI, 1),
        (InstructionKind::Load, 2),
        (InstructionKind::Store, 3),
        (InstructionKind::Cpy, 4),
        (InstructionKind::Jmp, 5),
        (InstructionKind::JmpC, 6),
        (InstructionKind::JmpZ, 7),
        (InstructionKind::Add, 8),
        (InstructionKind::Sub, 9),
        (InstructionKind::AddC, 12),
        (InstructionKind::SubC, 13),
        (InstructionKind::And, 15),
        (InstructionKind::LoadH, 22),
        (InstructionKind::LoadL, 23),
        (InstructionKind::StoreH, 24),
        (InstructionKind::StoreL, 25),
        (InstructionKind::Halt, 26),
    ]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstructionKind {
    Nop,

    LoadI,
    Load,
    Store,
    Cpy,

    Jmp,
    JmpC,
    JmpZ,

    Add,
    Sub,

    AddC,
    SubC,

    And,

    LoadH,
    LoadL,
    StoreH,
    StoreL,

    Halt,
}

impl InstructionKind {
    pub fn from_opcode(opcode: usize) -> Option<Self> {
        KIND_OPCODE_BIMAP.get_by_right(&opcode).copied()
    }

    pub fn opcode(&self) -> usize {
        *KIND_OPCODE_BIMAP
            .get_by_left(self)
            .expect("No opcode mapping for instruction kind")
    }

    pub fn has_immediate(&self) -> bool {
        match self {
            // I can't wait to debug for hours when I eventually add another
            // instruction with an immediate but forget to add it here.
            Self::LoadI => true,
            _ => false,
        }
    }
}

impl Display for InstructionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Nop => "nop",
            Self::LoadI => "loadi",
            Self::Load => "load",
            Self::Store => "store",
            Self::Cpy => "cpy",
            Self::Jmp => "jmp",
            Self::JmpC => "jmpc",
            Self::JmpZ => "jmpz",
            Self::Add => "add",
            Self::Sub => "sub",
            Self::AddC => "addc",
            Self::SubC => "subc",
            Self::And => "and",
            Self::LoadH => "loadh",
            Self::LoadL => "loadl",
            Self::StoreH => "storeh",
            Self::StoreL => "storel",
            Self::Halt => "halt",
        })
    }
}
