use std::{fs, path::PathBuf, process::exit};

use anyhow::anyhow;
use clap::Parser;
use command::{Command, CommandError};
use libemulator::Emulator;
use libstrmisa::{instruction::Instruction, Word};
use log::{error, info, LevelFilter};

mod command;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long, default_value_t = Word::MAX)]
    memory_size: Word,

    #[arg(short, long)]
    program_path: PathBuf,

    #[arg(long, default_value_t = { "".to_owned() })]
    log: String,
}

fn main() {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .parse_filters(&args.log)
        .init();

    if let Err(e) = Cli::new(args).map(|mut cli| cli.run()) {
        error!("Fatal: {}", e);
    }
}

struct Cli {
    args: Args,
    emulator: Emulator,
}

impl Cli {
    pub fn new(args: Args) -> anyhow::Result<Self> {
        let program =
            fs::read(&args.program_path).map_err(|e| anyhow!("Couldn't read program: {}", e))?;

        let emulator = Emulator::new(args.memory_size, program)?;

        Ok(Self { args, emulator })
    }

    pub fn run(&mut self) {
        loop {
            if let Err(e) = self.run_cmd() {
                match e.downcast::<CommandError>() {
                    Ok(CommandError::MissingArgument(1)) => {} // Missing command argument
                    Ok(e) => error!("Bad command: {}", e),
                    Err(e) => error!("{:?}", e),
                }
            }
        }
    }

    fn run_cmd(&mut self) -> anyhow::Result<()> {
        let cmd = Command::prompt()?;
        let mut cmd_args = cmd.args();

        match cmd_args.next()? {
            "p" | "print" => {
                let set_alu_flags = self
                    .emulator
                    .alu
                    .flags
                    .iter_names()
                    .filter(|(_, flag)| !flag.is_empty())
                    .map(|(name, _)| name.replace('"', ""))
                    .collect::<Vec<_>>();

                let instruction_deassembly = match self.deassemble_pc_instruction() {
                    Ok(s) => s,
                    Err(s) => s,
                };

                info!("PC:          {:05}", self.emulator.pc);
                info!("Deassembled: {}", instruction_deassembly);
                info!("Registers:   {:05?}", self.emulator.reg_file);
                info!("ALU flags:   {:?}", set_alu_flags);
            }

            "e" | "exec" | "execute" => {
                let instruction_count: usize = cmd_args.next_parsed().unwrap_or(Ok(1))?;

                for _ in 0..instruction_count {
                    self.emulator.execute_instruction()?;
                }
            }

            "q" | "quit" | "exit" => exit(0),

            _ => return Err(CommandError::UnknownCommand.into()),
        }

        Ok(())
    }

    fn deassemble_pc_instruction(&self) -> Result<String, String> {
        let instruction_word = self
            .emulator
            .memory
            .word(self.emulator.pc)
            .ok_or("<out of bounds>".to_string())?;

        let mut instruction = Instruction::deassemble_instruction_word(instruction_word)
            .map_err(|e| format!("<{}>", e))?;

        if instruction.kind.has_immediate() {
            let immediate = self
                .emulator
                .memory
                .word(self.emulator.pc + libstrmisa::BYTES_PER_WORD as Word)
                .ok_or("<immediate out of bounds>")?;

            instruction.immediate = Some(immediate);
        }

        Ok(format!("{}", instruction))
    }
}
