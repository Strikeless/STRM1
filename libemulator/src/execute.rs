use libisa::{
    instruction::{kind::InstructionKind, Instruction},
    Word,
};

use crate::{alu::flags::ALUFlags, volatile::mutcell::VolatileMutCell, Emulator, ExecuteErr, ExecuteOk};

impl Emulator {
    pub fn execute_parsed_instruction(
        &mut self,
        instruction: Instruction,
    ) -> Result<ExecuteOk, ExecuteErr> {
        match instruction.kind {
            InstructionKind::Nop => {}

            InstructionKind::LoadI => {
                let mut dest = self.reg_a_mut(&instruction);
                let value = instruction.immediate.unwrap();

                *dest = value;
            }

            InstructionKind::Load => {
                let src_addr = *self.reg_b(&instruction);
                let src_value = *self.mem_word_or_err(src_addr)?;

                let mut dest = self.reg_a_mut(&instruction);
                *dest = src_value;
            }

            InstructionKind::Store => {
                let dest_addr = *self.reg_a(&instruction);
                let src_value = *self.reg_b(&instruction);

                let mut dest_value = self.mem_word_mut_or_err(dest_addr)?;
                *dest_value = src_value;
            }

            InstructionKind::Cpy => {
                let src = *self.reg_b(&instruction);
                let mut dest = self.reg_a_mut(&instruction);
                *dest = src;
            }

            InstructionKind::Jmp => {
                let addr = *self.reg_a(&instruction);
                self.pc = addr;
            }

            InstructionKind::JmpC => {
                if self.alu.flags.contains(ALUFlags::CARRY) {
                    let addr = *self.reg_a(&instruction);
                    self.pc = addr;
                }
            }

            InstructionKind::JmpZ => {
                if self.alu.flags.contains(ALUFlags::ZERO) {
                    let addr = *self.reg_a(&instruction);
                    self.pc = addr;
                }
            }

            InstructionKind::Add => {
                let b = *self.reg_b(&instruction);
                let a = *self.reg_a(&instruction);

                let result = self.alu.add(a, b);
                *self.reg_a_mut(&instruction) = result;
            }

            InstructionKind::Sub => {
                let b = *self.reg_b(&instruction);
                let a = *self.reg_a(&instruction);

                let result = self.alu.sub(a, b);
                *self.reg_a_mut(&instruction) = result;
            }

            InstructionKind::AddC => {
                let b = *self.reg_b(&instruction);
                let a = *self.reg_a(&instruction);

                let result = self.alu.addc(a, b);
                *self.reg_a_mut(&instruction) = result;
            }

            InstructionKind::SubC => {
                let b = *self.reg_b(&instruction);
                let a = *self.reg_a(&instruction);

                let result = self.alu.subc(a, b);
                *self.reg_a_mut(&instruction) = result;
            }

            InstructionKind::And => {
                let b = *self.reg_b(&instruction);
                let a = *self.reg_a(&instruction);

                let result = self.alu.and(a, b);
                *self.reg_a_mut(&instruction) = result;
            }

            InstructionKind::LoadH => {
                let src_addr = *self.reg_b(&instruction);
                let src_value = *self.mem_byte_or_err(src_addr)?;

                let mut dest = self.reg_a_mut(&instruction);
                *dest = ((src_value as u16) << 8) | (*dest & 0x00FF);
            }

            InstructionKind::LoadL => {
                let src_addr = *self.reg_b(&instruction);
                let src_value = *self.mem_byte_or_err(src_addr)?;

                let mut dest = self.reg_a_mut(&instruction);
                *dest = (*dest & 0xFF00) | (src_value as u16)
            }

            InstructionKind::StoreH => {
                let src_value = *self.reg_b(&instruction);

                let dest_addr = *self.reg_a(&instruction);
                let mut dest_value = self.mem_byte_mut_or_err(dest_addr)?;

                *dest_value = ((src_value & 0xFF00) >> 8) as u8;
            }

            InstructionKind::StoreL => {
                let src_value = *self.reg_b(&instruction);

                let dest_addr = *self.reg_a(&instruction);
                let mut dest_value = self.mem_byte_mut_or_err(dest_addr)?;

                *dest_value = (src_value & 0x00FF) as u8;
            }

            InstructionKind::Halt => return Ok(ExecuteOk::Halted),
        }

        Ok(ExecuteOk::Normal)
    }

    fn reg_a(&self, instruction: &Instruction) -> &Word {
        self.reg_word(instruction.reg_a.unwrap())
    }

    fn reg_a_mut(&mut self, instruction: &Instruction) -> VolatileMutCell<'_, Word, usize> {
        self.reg_word_mut(instruction.reg_a.unwrap())
    }

    fn reg_b(&self, instruction: &Instruction) -> &Word {
        self.reg_word(instruction.reg_b.unwrap())
    }
}
