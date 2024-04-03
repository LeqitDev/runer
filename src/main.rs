use std::fs;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tabled::{
    settings::{style::HorizontalLine, Style},
    Table, Tabled,
};

#[derive(Parser)]
#[command(
    name = "runer",
    about = "A simple project-scoped command hub and runner",
    version = "0.1.0"
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize runer in the current directory
    Init {},
    /// Add a new command
    Add {
        /// Name of the command
        name: String,
        /// Command to run
        cmd: String,
        /// Description of the command
        desc: Option<String>,
    },
    /// Run a command
    Run { name: String },
    /// List all commands
    List {},
    /// Remove a command
    Remove {
        /// Name of the command to remove
        name: String,
    },
}

fn main() {
    let args = Args::parse();

    let mut commands = if fs::metadata(".projecto.json").is_ok() {
        collect_commands()
    } else {
        vec![]
    };

    match &args.command {
        Some(Commands::Init {}) => {
            // create .projecto.json file
            match save(commands) {
                Ok(_) => println!("Initialized successfully."),
                Err(e) => println!("Error initializing: {}", e),
            }
        }
        Some(Commands::Add { name, cmd, desc }) => {
            // if command already exists, update it
            if let Some(existing_command) =
                commands.iter_mut().find(|command| command.name == *name)
            {
                println!(
                    "Updating command: {}, from '{}' to '{}'",
                    name, existing_command.cmd, cmd
                );
                existing_command.cmd = cmd.clone();
                if let Some(desc) = desc {
                    println!("Updating description: {}", desc);
                    existing_command.desc = Some(desc.clone());
                }
            } else {
                println!("Adding command: {} '{}' {:?}", name, cmd, desc);
                commands.push(Command::new(name.clone(), cmd.clone(), desc.clone()));
            }
            save(commands).unwrap();
        }
        Some(Commands::Run { name }) => {
            println!("Running command: {}", name);
            let command = commands.iter().find(|command| command.name == *name);
            if let Some(command) = command {
                println!("Executing: {}", command.cmd);
                let output = if cfg!(target_os = "windows") {
                    // windows
                    std::process::Command::new("cmd")
                        .arg("/C")
                        .arg(&command.cmd)
                        .output()
                        .expect("failed to execute process")
                } else {
                    // unix
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&command.cmd)
                        .output()
                        .expect("failed to execute process")
                };
                println!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                println!("Command not found.");
            }
        }
        Some(Commands::List {}) => {
            println!("Listing commands...");
            if commands.is_empty() {
                println!("No commands found.");
            } else {
                let mut tbl = Table::new(&commands);
                let style = Style::modern()
                    .horizontals([(1, HorizontalLine::inherit(Style::modern()).horizontal('═'))])
                    // .verticals([(1, VerticalLine::inherit(Style::modern()))])
                    .remove_horizontal()
                    .remove_vertical();
                tbl.with(style);
                println!("{}", tbl);
            }
        }
        Some(Commands::Remove { name }) => {
            println!("Removing command: {}", name);
            commands.retain(|command| command.name != *name);
        }
        None => {
            println!("No command provided");
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Tabled)]
struct Command {
    name: String,
    cmd: String,
    #[tabled(display_with = "display_option")]
    desc: Option<String>,
}

fn display_option(o: &Option<String>) -> String {
    match o {
        Some(s) => s.clone(),
        None => "*No description*".to_string(),
    }
}

impl Command {
    fn new(name: String, cmd: String, desc: Option<String>) -> Command {
        Command { name, cmd, desc }
    }
}

fn collect_commands() -> Vec<Command> {
    let contents = match fs::read_to_string(".projecto.json") {
        Ok(contents) => contents,
        Err(e) => {
            println!("Error reading .projecto.json: {}", e);
            std::process::exit(1);
        }
    };

    let commands: Vec<Command> = match serde_json::from_str(&contents) {
        Ok(commands) => commands,
        Err(e) => {
            println!("Error parsing .projecto.json: {}", e);
            std::process::exit(1);
        }
    };
    commands
}

fn save(commands: Vec<Command>) -> std::io::Result<()> {
    let content = match serde_json::to_string_pretty(&commands) {
        Ok(content) => content,
        Err(e) => {
            println!("Error serializing commands: {}", e);
            std::process::exit(1);
        }
    };

    // save commands to .projecto.json
    fs::write(".projecto.json", content)
}
