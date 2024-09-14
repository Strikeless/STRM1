use std::collections::HashMap;

use allocator::Allocator;
use anyhow::anyhow;
use target::Instruction;

use crate::lir::{LIRInstruction, LIRVarKey};

mod allocator;
mod target;

pub struct Backend<'a> {
    ir: &'a [LIRInstruction],
    output: Vec<u8>,

    vars: HashMap<LIRVarKey, Var>,
    var_allocator: Allocator,

    // These reserved registers should for sure be replaced with some sort of var allocation priority
    // system to not have single-purpose registers but always have registers for dealing with memory.
    reserved_reg_a: usize,
    reserved_reg_b: usize,
    reserved_reg_addr: usize,

    reg_a: Option<usize>,
    reg_b: Option<usize>,
}

impl<'a> Backend<'a> {
    pub fn new(ir: &'a [LIRInstruction]) -> Self {
        let mut var_allocator = Allocator::new();

        Self {
            ir,
            output: Vec::new(),

            vars: HashMap::new(),
            reserved_reg_a: var_allocator.alloc_reg().unwrap(),
            reserved_reg_b: var_allocator.alloc_reg().unwrap(),
            reserved_reg_addr: var_allocator.alloc_reg().unwrap(),
            var_allocator,

            reg_a: None,
            reg_b: None,
        }
    }

    pub fn compile(mut self) -> anyhow::Result<Vec<u8>> {
        for ir_instruction in self.ir {
            self.compile_ir_instruction(&ir_instruction)?;
        }

        Ok(self.output)
    }

    fn compile_ir_instruction(&mut self, ir_instruction: &LIRInstruction) -> anyhow::Result<()> {
        match ir_instruction {
            LIRInstruction::InitVar(id) => {
                let var = self
                    .var_allocator
                    .alloc()
                    .map_err(|_| anyhow!("Out of target address space"))?;

                self.vars.insert(*id, var);
            }
            LIRInstruction::DropVar(id) => {
                let var = self
                    .vars
                    .remove(id)
                    .ok_or_else(|| anyhow!("DropVar on non-existing var"))?;

                self.var_allocator.free(var);
            }
            LIRInstruction::ConstantA(value) => {
                self.reg_a = Some(self.reserved_reg_a);
                
                self.push_instruction(Instruction::LoadI {
                    dest: self.reg_a.unwrap(),
                    value: *value,
                });
            },
            LIRInstruction::LoadA(id) => {
                let var = self
                    .vars
                    .get(id)
                    .ok_or_else(|| anyhow!("LoadA on non-existing var"))?;

                match var {
                    // Even though we're loading from a real register, virtual register A may be mutated so a copy
                    // must be made to not unintentionally mutate this register as an operand.
                    Var::Register(reg) => self.push_instruction(Instruction::Cpy {
                        dest: self.reserved_reg_a,
                        src: *reg,
                    }),

                    Var::Memory(addr) => {
                        self.push_instruction(Instruction::LoadI {
                            dest: self.reserved_reg_addr,
                            value: *addr as u16,
                        });

                        self.push_instruction(Instruction::Load {
                            dest: self.reserved_reg_a,
                            src_addr: self.reserved_reg_addr,
                        });
                    }
                }

                self.reg_a = Some(self.reserved_reg_a);
            }
            LIRInstruction::LoadB(id) => {
                let var = self
                    .vars
                    .get(id)
                    .ok_or_else(|| anyhow!("LoadB on non-existing var"))?;

                match var {
                    // Unlike virtual register A, virtual register B cannot be mutated, so a
                    // LoadVar to it from a register is a no-op, no need for output instructions.
                    Var::Register(reg) => self.reg_b = Some(*reg),

                    Var::Memory(addr) => {
                        self.push_instruction(Instruction::LoadI {
                            dest: self.reserved_reg_addr,
                            value: *addr as u16,
                        });

                        self.push_instruction(Instruction::Load {
                            dest: self.reserved_reg_b,
                            src_addr: self.reserved_reg_addr,
                        });

                        self.reg_b = Some(self.reserved_reg_b);
                    }
                }
            }
            LIRInstruction::StoreA(id) => {
                let var = self
                    .vars
                    .get(id)
                    .ok_or_else(|| anyhow!("StoreA on non-existing var"))?;

                let src = self
                    .reg_a
                    .ok_or_else(|| anyhow!("StoreA before A was loaded"))?;

                match var {
                    Var::Register(reg) => {
                        self.push_instruction(Instruction::Cpy { dest: *reg, src })
                    }

                    Var::Memory(addr) => {
                        self.push_instruction(Instruction::LoadI {
                            dest: self.reserved_reg_addr,
                            value: *addr as u16,
                        });

                        self.push_instruction(Instruction::Store {
                            dest_addr: self.reserved_reg_addr,
                            src,
                        });
                    }
                }
            }
            LIRInstruction::Add => self.push_instruction(Instruction::Add {
                a: self
                    .reg_a
                    .ok_or_else(|| anyhow!("Sum before A was loaded"))?,
                b: self
                    .reg_b
                    .ok_or_else(|| anyhow!("Sum before B was loaded"))?,
            }),
            LIRInstruction::Sub => self.push_instruction(Instruction::Sub {
                a: self
                    .reg_a
                    .ok_or_else(|| anyhow!("Sum before A was loaded"))?,
                b: self
                    .reg_b
                    .ok_or_else(|| anyhow!("Sum before B was loaded"))?,
            }),
        }

        Ok(())
    }

    fn push_instruction(&mut self, instruction: Instruction) {
        self.output.append(&mut instruction.build());
    }
}

enum Var {
    Register(usize),
    Memory(usize),
}
