use anyhow::anyhow;
use kind::InstructionKind;

use crate::{Immediate, Register, Word};

pub mod kind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub reg_a: Option<Register>,
    pub reg_b: Option<Register>,
    pub immediate: Option<Immediate>,
}

impl Instruction {
    pub fn new(kind: InstructionKind) -> Self {
        Self {
            kind,
            reg_a: None,
            reg_b: None,
            immediate: None,
        }
    }

    pub fn with_reg_a(mut self, reg_a: Register) -> Self {
        self.reg_a = Some(reg_a);
        self
    }

    pub fn with_reg_b(mut self, reg_b: Register) -> Self {
        self.reg_b = Some(reg_b);
        self
    }

    pub fn with_immediate(mut self, immediate: Immediate) -> Self {
        self.immediate = Some(immediate);
        self
    }

    // TODO: Assemble and deassemble should be using concrete error types instead of anyhow.

    pub fn assemble(self) -> anyhow::Result<Vec<u8>> {
        let has_immediate = self.kind.has_immediate();
        let mut output = Vec::with_capacity(if has_immediate { crate::BYTES_PER_WORD * 2 } else { crate::BYTES_PER_WORD });

        output.extend(crate::word_to_bytes(
            (
                self.kind.opcode() << 10
                | self.reg_a.unwrap_or(0) << 6
                | self.reg_b.unwrap_or(0) << 2
            ) as u16
        ));

        if has_immediate {
            let immediate = self.immediate
                .ok_or_else(|| anyhow!("Instruction expected immediate"))?;

            output.extend(crate::word_to_bytes(immediate));
        }

        Ok(output)
    }

    pub fn deassemble_instruction_word(instruction: Word) -> anyhow::Result<Self> {
        let [opcode, reg_a, reg_b] = [
            (instruction >> 10) as usize,
            (instruction >> 6) as usize & 0xF,
            (instruction >> 2) as usize & 0xF,
        ];

        let kind = InstructionKind::from_opcode(opcode)
            .ok_or_else(|| anyhow!("Unrecognized opcode"))?;

        Ok(Self {
            kind,
            // FIXME: Registers should be none if the instruction kind doesn't use them.
            reg_a: Some(reg_a),
            reg_b: Some(reg_b),
            immediate: None,
        })
    }
}
