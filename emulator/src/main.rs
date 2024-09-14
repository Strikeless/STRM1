#![feature(int_roundings)]

use std::{error::Error, fmt::Debug, fs, io, path::PathBuf, process::exit, str::FromStr, time::{Duration, SystemTime}};

use byteorder::{BigEndian, LittleEndian};
use clap::Parser;
use emulator::Emulator;
use endian::EndianRewriteExt;

mod emulator;
mod endian;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long)]
    bios_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let bios = match fs::read(args.bios_path) {
        Ok(bios) => bios,
        Err(e) => {
            eprintln!("Failed to read bios binary: {}", e);
            exit(1);
        }
    };

    let mut emulator = Emulator::new(&bios);
    emulate(&mut emulator);
}

fn emulate(emulator: &mut Emulator) {
    let mut last_cmd_nanos = Duration::ZERO;

    loop {

        println!(
            "<<<   PC: {}, GPRs: {:?}, {} ms ({} ns)   >>>",
            emulator.program_counter, emulator.gpr_file,
            last_cmd_nanos.as_millis(),
            last_cmd_nanos.as_nanos(),
        );

        let cmd = io::stdin().lines().next().unwrap().unwrap();
        let cmd = cmd.split(' ');

        let start_time = SystemTime::now();

        if let Err(e) = execute_command(cmd, emulator) {
            eprintln!("!> {}", e);
        }

        let finish_time = SystemTime::now();
        last_cmd_nanos = finish_time.duration_since(start_time).unwrap_or(Duration::ZERO);
    }
}

fn execute_command<'a, I>(mut args: I, emulator: &mut Emulator) -> Result<(), Box<dyn Error>>
where
    I: Iterator<Item = &'a str>,
{
    fn parse<'a, I, T>(cmd: &mut I) -> Option<T>
    where
        I: Iterator<Item = &'a str>,
        T: FromStr + Default,
        <T as FromStr>::Err: Debug,
    {
        cmd.next().map(|s| s.parse().ok())?
    }

    match args
        .find(|item| !item.is_empty() && *item != " ")
        .ok_or("No command specified")?
    {
        "e" => {
            let instruction_count = parse(&mut args).unwrap_or(1);

            for _ in 0..instruction_count {
                emulator.execute_instruction();
            }
        }
        "eb" => {
            let break_pc = parse(&mut args).ok_or("No break PC given")?;
            let instruction_limit = parse(&mut args).unwrap_or(1000000);

            let mut executed_instructions = 0;
            while emulator.program_counter != break_pc {
                if executed_instructions >= instruction_limit {
                    eprintln!(
                        "Didn't reach breakpoint by {} executed instructions",
                        executed_instructions
                    );
                    return Ok(());
                }

                emulator.execute_instruction();
                executed_instructions += 1;
            }

            println!("Executed {} instructions", executed_instructions);
        }
        "er" => {
            let break_register: usize = parse(&mut args).ok_or("No break register given")?;
            let break_value = parse(&mut args).ok_or("No break value given")?;

            let instruction_limit = parse(&mut args).unwrap_or(1000000);

            let mut executed_instructions = 0;
            while emulator.gpr_file[break_register] != break_value {
                if executed_instructions >= instruction_limit {
                    eprintln!(
                        "Didn't reach break value by {} executed instructions",
                        executed_instructions
                    );
                    return Ok(());
                }

                emulator.execute_instruction();
                executed_instructions += 1;
            }

            println!("Executed {} instructions", executed_instructions);
        }
        "d" => {
            let begin = parse(&mut args).unwrap_or(0);
            let length = parse(&mut args).unwrap_or(emulator::MEMORY_SIZE);
            let data_mode = args.next().unwrap_or("w");

            match data_mode {
                "w" => {
                    let dump = (begin..begin + length * 2).step_by(emulator::AUS_PER_WORD as usize)
                        .map(|addr| *emulator.memory(addr as u16))
                        .collect::<Vec<_>>();

                    println!("{:?}", dump);
                }
                "b" => {
                    let dump = (begin..begin + length)
                        .map(|addr| *emulator.memory(addr as u16))
                        .map(|word| (word >> 8) as u8)
                        .collect::<Vec<_>>();

                    println!("{:?}", dump);
                }
                "a" => {
                    let dump = (begin..begin + length)
                        .map(|addr| *emulator.memory(addr as u16))
                        .map(|word| (word >> 8) as u8)
                        .map(|byte| byte as char)
                        .collect::<String>();

                    println!("{:?}", dump);
                }
                _ => Err("Unrecognized dump mode")?,
            }
        }
        "cd" => {
            let decimal: usize = parse(&mut args).ok_or("Invalid decimal")?;
            let format = args.next().ok_or("No format specified")?;

            match format {
                "x" => println!("{:x}", decimal),
                "b" => println!("{:b}", decimal),
                _ => Err("Unrecognized format")?,
            }
        }
        "rem" => {
            let target_endian = args.next().ok_or("No target endian given")?;

            let rewritten = match target_endian {
                "g" => emulator.memory.endian_rewrite_to_guest(),
                "h" => emulator.memory.endian_rewrite_to_host(),
                "l" => emulator.memory.endian_rewrite::<BigEndian, LittleEndian>(),
                "b" => emulator.memory.endian_rewrite::<LittleEndian, BigEndian>(),
                _ => Err("Unrecognized target endian")?,
            };

            emulator.memory = rewritten.try_into().unwrap();
        }
        "jmp" => {
            emulator.program_counter = parse(&mut args).ok_or("No address given")?;
        }

        "q" => exit(0),
        _ => eprintln!("Unrecognized command!"),
    }

    let unused_arg_count = args.count();
    if unused_arg_count != 0 {
        eprintln!("{} unused command arguments!", unused_arg_count);
    }

    Ok(())
}
