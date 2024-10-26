use std::{fs, path::PathBuf, process::exit};

use anyhow::anyhow;
use clap::{builder::Str, Parser};
use command::{Command, CommandError};
use libdeassembler::Deassembler;
use libemulator::{tracing::none::NoTraceData, Emulator};
use libisa::{instruction::Instruction, Word};
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
    emulator: Emulator<NoTraceData>,
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
                    Ok(CommandError::MissingArgument(1)) => {} // Missing first argument (command)
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
            "e" | "exec" | "execute" => {
                let instruction_count: usize = cmd_args.next_parsed().unwrap_or(Ok(1))?;

                for _ in 0..instruction_count {
                    self.emulator.execute_instruction()?;
                }
            }

            "p" | "print" => {
                let set_alu_flags = self
                    .emulator
                    .alu
                    .flags
                    .iter_names()
                    .filter(|(_, flag)| !flag.is_empty())
                    .map(|(name, _)| name.replace('"', ""))
                    .collect::<Vec<_>>();

                info!("PC:          {:05}", self.emulator.pc);
                info!("Deassembled: {}", self.deassemble_pc_instruction());
                info!(
                    "Registers:   {:05?}",
                    self.emulator.reg_file.array_clone_untraced()
                );
                info!("ALU flags:   {:?}", set_alu_flags);
            }

            "d" | "dump" => {
                let addr: Word = cmd_args.next_parsed()??;
                let len: Word = cmd_args.next_parsed()??;

                let words = (addr..addr + len)
                    .step_by(2)
                    .map(|addr| self.emulator.memory.word(addr).unwrap_or(0))
                    .collect::<Vec<_>>();

                let bytes = (addr..addr + len)
                    .map(|addr| self.emulator.memory.byte(addr).unwrap_or(0))
                    .collect::<Vec<_>>();

                let output = match cmd_args.next()? {
                    "db" | "decb" => format!("{:?}", bytes),
                    "dw" | "decw" => format!("{:?}", words),
                    "xb" | "hexb" => format!("{:x?}", bytes),
                    "xw" | "hexw" => format!("{:x?}", words),
                    // No binary because apparently {:b?} wont do for whatever reason.
                    "s" | "utf8" => String::from_utf8_lossy(&bytes).to_string(),
                    _ => {
                        return Err(CommandError::ParseError(
                            "Output format should be [d]ec(b/w), he[x](b/w), or utf8 [s]"
                                .to_string(),
                        )
                        .into())
                    }
                };

                info!("Dump {}..{}: {}", addr, addr + len, output);
            }

            "j" | "jmp" | "goto" => {
                let addr = cmd_args.next_parsed()??;
                self.emulator.pc = addr;
            }

            "q" | "quit" | "exit" => exit(0),

            _ => return Err(CommandError::Other("Unknown command".to_string()).into()),
        }

        Ok(())
    }

    fn deassemble_pc_instruction(&self) -> String {
        let mut deassembler = Deassembler::new(
            self.emulator
                .memory
                .iter_untraced()
                .skip(self.emulator.pc as usize),
        );

        deassembler.deassemble_instruction_ignorant()
    }
}
