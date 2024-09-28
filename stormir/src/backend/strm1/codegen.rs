use std::collections::HashMap;

use anyhow::{anyhow, Context};
use super::target::Instruction;
use super::var::{Var, VarAllocator, VarKey};

use crate::{ir::LirOp, transformer::Transformer};

pub struct CodegenTransformer {
    allocator: VarAllocator,
    vars: HashMap<VarKey, Var>,
    acc_reg: Option<usize>,
    sec_reg: Option<usize>,
}

impl CodegenTransformer {
    pub fn new() -> Self {
        Self {
            allocator: VarAllocator::new(),
            vars: HashMap::new(),
            acc_reg: None,
            sec_reg: None,
        }
    }
}

impl Transformer for CodegenTransformer {
    type Input = LirOp;
    type Output = Instruction;

    fn prepass(&mut self, input: &Vec<Self::Input>) -> anyhow::Result<()> {
        for (op_index, op) in input.iter().enumerate() {
            self.prepass_op(op_index, op).context(format!("prepass op {} ({:?})", op_index, op))?;
        }

        Ok(())
    }

    fn transform(&mut self, input: Vec<Self::Input>) -> anyhow::Result<Vec<Self::Output>> {
        let mut output = Vec::new();
        
        for (op_index, op) in input.iter().enumerate() {
            self.transform_op(&mut output, op_index, op).context(format!("transform op {} ({:?})", op_index, op))?;
        }

        Ok(output)
    }
}

impl CodegenTransformer {
    fn prepass_op(&mut self, op_index: usize, op: &LirOp) -> anyhow::Result<()> {
        match *op {
            LirOp::InitVar(id) => self.allocator.allocate(VarKey::Code(id), 0)?,
            LirOp::DropVar(id) => self.allocator.free(VarKey::Code(id))?,

            // Accumulator loads always need a register to store the mutable value in.
            LirOp::LoadAccumulator(..) => {
                self.allocator.allocate(VarKey::Compiler(op_index), usize::MAX)?
            }

            // Secondary loads need a register to store the value in if the load is from memory.
            LirOp::LoadSecondary(id) if self.var(VarKey::Code(id))?.in_memory() => {
                self.allocator.allocate(VarKey::Compiler(op_index), usize::MAX)?
            }

            // Accumulator stores need a register for the memory address if storing to memory.
            LirOp::StoreAccumulator(id) if self.var(VarKey::Code(id))?.in_memory() => {
                self.allocator.allocate(VarKey::Compiler(op_index), usize::MAX)?
            }

            // Constant load to accumulator always needs a register to store the mutable value in,
            // just like accumulator loads from variables.
            LirOp::LoadConstant(..) => {
                self.allocator.allocate(VarKey::Compiler(op_index), usize::MAX)?
            }

            _ => {}
        }

        self.vars = self.allocator.vars();
        Ok(())
    }

    fn transform_op(&mut self, output: &mut Vec<Instruction>, op_index: usize, op: &LirOp) -> anyhow::Result<()> {
        match *op {
            LirOp::InitVar(..) => {}
            LirOp::DropVar(..) => {}

            LirOp::LoadAccumulator(id) => {
                let src_var = *self.var(VarKey::Code(id))?;
                self.acc_reg = Some(self.compiler_reg(op_index)?);

                match src_var {
                    // Accumulator load from a register can be a simple register copy.
                    Var::Register { reg: var_reg, .. } => output.push(Instruction::Cpy {
                        dest: self.acc_reg.unwrap(),
                        src: var_reg,
                    }),

                    Var::Memory { addr } => output.extend([
                        // Load the memory address to the accumulator temporarily...
                        Instruction::LoadI {
                            dest: self.acc_reg.unwrap(),
                            value: addr as u16,
                        },
                        // ... and then load the actual value from memory to the accumulator.
                        Instruction::Load {
                            dest: self.acc_reg.unwrap(),
                            src_addr: self.acc_reg.unwrap(),
                        },
                    ]),
                }
            },

            LirOp::LoadSecondary(id) => {
                let src_var = self.var(VarKey::Code(id))?;

                match *src_var {
                    // Secondary load from a register can be a no-operation, 
                    // as the secondary register should never be mutated directly.
                    Var::Register { reg: var_reg, .. } => self.sec_reg = Some(var_reg),

                    // Same as accumulator load.
                    Var::Memory { addr } => {
                        self.sec_reg = Some(self.compiler_reg(op_index)?);

                        output.extend([
                            // Load the memory address to the secondary register temporarily...
                            Instruction::LoadI {
                                dest: self.sec_reg.unwrap(),
                                value: addr as u16,
                            },
                            // ... and then load the actual value from memory to the secondary register.
                            Instruction::Load {
                                dest: self.sec_reg.unwrap(),
                                src_addr: self.sec_reg.unwrap(),
                            },
                        ])
                    },
                }
            },

            LirOp::StoreAccumulator(id) => {
                let dest_var = self.var(VarKey::Code(id))?;
                let acc_reg = self.accumulator()?;

                match *dest_var {
                    Var::Register { reg: var_reg, .. } => output.push(Instruction::Cpy {
                        dest: var_reg,
                        src: acc_reg,
                    }),
                    Var::Memory { addr } => {
                        let temp_reg = self.compiler_reg(op_index)?;

                        output.extend([
                            Instruction::LoadI {
                                dest: temp_reg,
                                value: addr as u16,
                            },
                            Instruction::Store {
                                dest_addr: acc_reg,
                                src: acc_reg,
                            }
                        ])
                    },
                }
            },

            LirOp::LoadConstant(value) => {
                self.acc_reg = Some(self.compiler_reg(op_index)?);

                output.push(Instruction::LoadI {
                    dest: self.acc_reg.unwrap(),
                    value,
                });
            },

            LirOp::Add => output.push(Instruction::Add {
                a: self.accumulator()?,
                b: self.secondary()?,
            }),

            LirOp::Sub => output.push(Instruction::Add {
                a: self.accumulator()?,
                b: self.secondary()?,
            }),

            LirOp::InitLabel(id) => todo!(),
            LirOp::Goto(id) => todo!(),

            LirOp::Finish => output.push(Instruction::Halt),
        }

        Ok(())
    }

    fn var(&self, key: VarKey) -> anyhow::Result<&Var> {
        self.vars
            .get(&key)
            .ok_or_else(|| anyhow!("Tried to access undeclared var"))
    }

    fn compiler_reg(&self, op_index: usize) -> anyhow::Result<usize> {
        let Var::Register { reg, .. } = self.var(VarKey::Compiler(op_index)).context("No such compiler register variable")? else {
            panic!();
        };

        Ok(*reg)
    }

    fn accumulator(&self) -> anyhow::Result<usize> {
        self.acc_reg.context("Accumulator access before load")
    }

    fn secondary(&self) -> anyhow::Result<usize> {
        self.sec_reg.context("Secondary register access before load")
    }
}
