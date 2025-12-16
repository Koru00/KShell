use std::collections::HashMap;
use std::env::{self, Args};
use std::fmt::format;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

mod shell;
mod file_reader;

fn current_dir_string() -> Result<String, String> {
    match env::current_dir() {
        Ok(path) => Ok(path.to_string_lossy().into_owned()),
        Err(e) => Err(format!("Error in obtaing the current directory: {}", e)),
    }
}

fn main() {
    let mut shell = shell::Shell::new();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 { 
        loop {
            match current_dir_string() {
                Ok(dir) => print!("{}> ", dir),
                Err(err) => {
                    eprintln!("{}", err);
                    break;
                }
            }

            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                eprintln!("Error while reading the input");
                continue;
            }

            let input = input.trim();

            if input == "exit" {
                break;
            }

            shell.run_command(input);
        }
    }
    else {
        let filename = &args[1];
        match file_reader::read_file_lines(filename) {
            Ok(lines) => {
                for line in lines {
                    let line = line.trim();
                    shell.run_command(line);
                }
            }
            Err(e) => eprintln!("Error in the opening of the file: {}", filename),
        }
    }
}
