use std::iter::Peekable;

use libisa::{instruction::Instruction, Word};

pub struct Deassembler<'a, I>
where
    I: Iterator<Item = &'a u8>,
{
    code_iter: Peekable<I>,
}

impl<'a, I> Deassembler<'a, I>
where
    I: Iterator<Item = &'a u8>,
{
    pub fn new(code_iter: I) -> Self {
        Self {
            code_iter: code_iter.peekable(),
        }
    }

    pub fn deassemble(mut self) -> Result<Vec<Instruction>, String> {
        let mut output = Vec::new();

        while self.code_iter.peek().is_some() {
            output.push(self.deassemble_instruction()?);
        }

        Ok(output)
    }

    pub fn deassemble_text(self) -> String {
        match self.deassemble() {
            Ok(deassembly) => deassembly
                .iter()
                .map(|instr| format!("{}", instr))
                .collect(),

            Err(e) => e,
        }
    }

    pub fn deassemble_instruction(&mut self) -> Result<Instruction, String> {
        if self.code_iter.peek().is_none() {
            return Err("<out of deassembler bounds>".to_string());
        }

        let instruction_word = self
            .next_word()
            .ok_or("<incomplete instruction>".to_string())?;

        let mut instruction = Instruction::deassemble_instruction_word(instruction_word)
            .map_err(|e| format!("<{}>", e))?;

        if instruction.kind.has_immediate() {
            let immediate = self.next_word().ok_or("<incomplete immediate>")?;

            instruction.immediate = Some(immediate);
        }

        Ok(instruction)
    }

    pub fn deassemble_instruction_text(&mut self) -> String {
        match self.deassemble_instruction() {
            Ok(instr) => format!("{}", instr),
            Err(e) => e,
        }
    }

    fn next_word(&mut self) -> Option<Word> {
        let first_byte = *self.code_iter.next()?;
        let second_byte = *self.code_iter.next()?;
        Some(libisa::bytes_to_word([first_byte, second_byte]))
    }
}
