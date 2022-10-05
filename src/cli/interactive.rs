pub use clap::{Args, Parser, Subcommand};
use rustyline::{error::ReadlineError, Editor};

use super::{logs::Logs, stop::Stop, CliCommand};

const LENRA_COMMAND: &str = "lenra";

pub fn run_interactive_command() -> Result<(), ReadlineError> {
    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
                let args = &mut line.split_whitespace().collect::<Vec<&str>>();

                let first_arg = if args.len() > 0 { Some(args[0]) } else { None };
                if let Some(arg) = first_arg {
                    if LENRA_COMMAND != arg {
                        args.push(LENRA_COMMAND);
                        args.rotate_right(1);
                    }
                }

                let interactive = Interactive::parse_from(args.clone());
                interactive.command.run();
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt")
}

/// The Lenra interactive command line interface
#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Interactive {
    #[clap(subcommand)]
    pub command: InteractiveCommand,
}

/// The interactive commands
#[derive(Subcommand)]
pub enum InteractiveCommand {
    /// View output from the containers
    Logs(Logs),
    /// Stop your app previously started with the start command
    Stop(Stop),
}

impl CliCommand for InteractiveCommand {
    fn run(&self) {
        match self {
            InteractiveCommand::Logs(logs) => println!("logs"),//logs.run(),
            InteractiveCommand::Stop(stop) => println!("stop"),//stop.run(),
        };
    }
}
