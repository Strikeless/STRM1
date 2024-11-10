use std::{env, fs, path::PathBuf, process::exit};

use libdeassembler::Deassembler;

fn main() {
    let path: PathBuf = env::args().skip(1).collect();

    if path.file_name().is_none() {
        eprintln!("Specify the program file path as arguments.");
        exit(1);
    }

    let program = match fs::read(path) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Error reading program file: {}", e);
            exit(1);
        }
    };

    let deassembler = Deassembler::new(program.iter());
    println!("{}", deassembler.deassemble_text());
    println!();
}
