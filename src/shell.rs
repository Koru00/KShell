use std::collections::HashMap;
use std::env::{self, Args};
use std::fmt::format;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, CommandArgs};

pub trait Command {
    fn name(&self) -> &'static str;
    fn syntax(&self) -> &'static str { "" }
    fn execute(&self, shell: &Shell, args: &[String]) -> Result<(), String>;
}

pub struct CdCommand;
impl Command for CdCommand {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn syntax(&self) -> &'static str {
        "cd [[-Path] <string>]"
    }

    fn execute(&self, Shell: &Shell, args: &[String]) -> Result<(), String> {
        if args.is_empty() {
            return Err("cd need a directory".into());
        }
        env::set_current_dir(&args[0]).map_err(|e| format!("Error while chaging directory: {}", e))
    }
}

pub struct EchoCommand;
impl Command for EchoCommand {
    fn name(&self) -> &'static str { "echo" }

    fn syntax(&self) -> &'static str {
        "echo [-InputObject] <psobject[]>"
    }

    fn execute(&self, shell: &Shell, args: &[String]) -> Result<(), String> {
        let expanded: Vec<String> = args.iter().map(|arg| {
            if arg == "$?" {
                shell.last_status.to_string()
            } else {
                arg.clone()
            }
        }).collect();

        println!("{}", expanded.join(" "));
        Ok(())
    }
}


pub struct TypeCommand;
impl Command for TypeCommand {
    fn name(&self) -> &'static str {
        "type"
    }

    fn syntax(&self) -> &'static str {
        "type [[-Path] <string>]"
    }

    fn execute(&self, shell: &Shell, args: &[String]) -> Result<(), String> {
        if args.is_empty() {
            return Err("Error: Type need the name of a command".into());
        }

        let cmd_name = &args[0];

        if shell.commands.contains_key(cmd_name.as_str()) {
            println!("{} is a build-in command", cmd_name);
            return Ok(());
        }

        if let Some(paths) = env::var_os("PATH") {
            for dir in env::split_paths(&paths) {
                let mut candidate = PathBuf::from(&dir);
                candidate.push(cmd_name);
                if candidate.exists() {
                    println!("{} is {}", cmd_name, candidate.display());
                    return Ok(());
                }
            }
        }

        println!("{} not found", cmd_name);
        Ok(())
    }
}

pub struct ExecCommand;
impl Command for ExecCommand {
    fn name(&self) -> &'static str {
        "exec"
    }

    fn syntax(&self) -> &'static str {
        "exec [[-ProgramName] <string>] [[-Args] <string[]>]"
    }

    fn execute(&self, _shell: &Shell, args: &[String]) -> Result<(), String> {
        if args.is_empty() {
            return Err("exec needs a program to execute".into());
        }

        let program = &args[0];
        let prog_args = &args[1..];

        let path_to_exec = if program.contains('/') || program.contains('\\') {
            PathBuf::from(program)
        } else {
            let paths = env::var_os("PATH").ok_or("PATH not set")?;
            let exts = env::var("PATHEXT").unwrap_or(".EXE;.BAT;.CMD".into());
            let exts: Vec<&str> = exts.split(';').collect();

            let mut found: Option<PathBuf> = None;
            for dir in env::split_paths(&paths) {
                for ext in &exts {
                    let candidate = dir.join(format!("{}{}", program, ext));
                    if candidate.exists() {
                        found = Some(candidate);
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }

            found.ok_or_else(|| format!("{}: command not found in PATH", program))?
        };

        let mut child = ProcessCommand::new(path_to_exec)
            .args(prog_args)
            .spawn()
            .map_err(|e| format!("Error starting {}: {}", program, e))?;

        child
            .wait()
            .map_err(|e| format!("Error while waiting {}: {}", program, e))?;

        Ok(())
    }
}

pub struct HelpCommand;
impl Command for HelpCommand {
    fn name(&self) -> &'static str {
        "help"
    }

    fn syntax(&self) -> &'static str {
        "help [[-Name] <string>]"
    }

    fn execute(&self, shell: &Shell, args: &[String]) -> Result<(), String> {
        if args.is_empty() {
            println!("Available commands:");
            for (name, cmd) in &shell.commands {
                println!("  {:<10} {}", name, cmd.syntax());
            }
            return Ok(());
        }

        let cmd_name = &args[0];
        match shell.commands.get(cmd_name.as_str()) {
            Some(cmd) => {
                println!("{}: {}", cmd.name(), cmd.syntax());
                return Ok(());
            }
            None => Err(format!("no such command '{}'", cmd_name)),
        }
    }
}

pub struct Shell {
    commands: HashMap<&'static str, Box<dyn Command>>,
    last_status: i32,
}

impl Shell {
    pub fn new() -> Self {
        let mut commands: HashMap<&'static str, Box<dyn Command>> = HashMap::new();
        commands.insert(CdCommand.name(), Box::new(CdCommand));
        commands.insert(EchoCommand.name(), Box::new(EchoCommand));
        commands.insert(TypeCommand.name(), Box::new(TypeCommand));
        commands.insert(ExecCommand.name(), Box::new(ExecCommand));
        commands.insert(HelpCommand.name(), Box::new(HelpCommand));

        Shell {
            commands,
            last_status: 0,
        }
    }

    pub fn run_command(&mut self, input: &str) {
        let parts: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
        if parts.is_empty() {
            return;
        }

        let cmd_name = &parts[0];
        let args = &parts[1..];

        match self.commands.get(cmd_name.as_str()) {
            Some(cmd) => match cmd.execute(self, args) {
                Ok(()) => self.last_status = 0,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    self.last_status = 1;
                }
            },
            None => {
                if let Some(exec_cmd) = self.commands.get("exec") {
                    let mut full_args = vec![cmd_name.clone()];
                    full_args.extend_from_slice(args);

                    match exec_cmd.execute(self, &full_args) {
                        Ok(()) => self.last_status = 0,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            self.last_status = 1;
                        }
                    }
                } else {
                    eprintln!("{}: command not found", cmd_name);
                    self.last_status = 127;
                }
            }
        }
    }
}
