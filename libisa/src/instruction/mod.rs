use std::fmt::Display;

use kind::InstructionKind;
use thiserror::Error;

use crate::{Immediate, Register, Word};

pub mod assembler;
pub mod kind;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum AssemblyError {
    #[error("Missing immediate")]
    MissingImmediate,
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum InstructionDeassemblyError {
    #[error("Unrecognized opcode")]
    UnrecognizedOpcode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub reg_a: Option<Register>,
    pub reg_b: Option<Register>,
    pub immediate: Option<Immediate>,
}

impl Instruction {
    pub const fn new(kind: InstructionKind) -> Self {
        Self {
            kind,
            reg_a: None,
            reg_b: None,
            immediate: None,
        }
    }

    pub const fn with_reg_a(mut self, reg_a: Register) -> Self {
        self.reg_a = Some(reg_a);
        self
    }

    pub const fn with_reg_b(mut self, reg_b: Register) -> Self {
        self.reg_b = Some(reg_b);
        self
    }

    pub const fn with_immediate(mut self, immediate: Immediate) -> Self {
        self.immediate = Some(immediate);
        self
    }

    pub fn assemble(self) -> Result<Vec<u8>, AssemblyError> {
        let has_immediate = self.kind.has_immediate();
        let mut output = Vec::with_capacity(if has_immediate {
            crate::BYTES_PER_WORD * 2
        } else {
            crate::BYTES_PER_WORD
        });

        output.extend(crate::word_to_bytes(
            (self.kind.opcode() << 10 | self.reg_a.unwrap_or(0) << 6 | self.reg_b.unwrap_or(0) << 2)
                as u16,
        ));

        if has_immediate {
            let immediate = self.immediate.ok_or(AssemblyError::MissingImmediate)?;

            output.extend(crate::word_to_bytes(immediate));
        }

        Ok(output)
    }

    pub fn deassemble_instruction_word(
        instruction: Word,
    ) -> Result<Self, InstructionDeassemblyError> {
        let [opcode, reg_a, reg_b] = [
            (instruction >> 10) as usize,
            (instruction >> 6) as usize & 0xF,
            (instruction >> 2) as usize & 0xF,
        ];

        let kind = InstructionKind::from_opcode(opcode)
            .ok_or(InstructionDeassemblyError::UnrecognizedOpcode)?;

        Ok(Self {
            kind,
            // Don't set registers if they're zero and the instruction doesn't use them.
            reg_a: (reg_a != 0 || kind.has_reg_a()).then_some(reg_a),
            reg_b: (reg_b != 0 || kind.has_reg_b()).then_some(reg_b),
            immediate: None,
        })
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.kind))?;

        if let Some(reg_a) = self.reg_a {
            f.write_fmt(format_args!(" %{}", reg_a))?;
        }

        if let Some(reg_b) = self.reg_b {
            f.write_fmt(format_args!(", %{}", reg_b))?;
        }

        if let Some(immediate) = self.immediate {
            f.write_fmt(format_args!(", ${}", immediate))?;
        }

        Ok(())
    }
}
