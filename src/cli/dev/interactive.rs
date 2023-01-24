use super::terminal::{DevCli, DevTermCommand};
use crate::{
    errors::Result,
    keyboard_event::{keyevent_to_string, KeyEventListener, KeyboardListener},
};
use clap::CommandFactory;
pub use clap::{Args, Parser, Subcommand};
use colored::{Color, Colorize};
use lazy_static::__Deref;
use log::debug;
use rustyline::{Cmd, KeyCode, KeyEvent, Modifiers, Movement};
use std::sync::{Arc, Mutex};
use strum::{Display, EnumIter, IntoEnumIterator};

const ENTER_EVENT: KeyEvent = KeyEvent(KeyCode::Enter, Modifiers::NONE);

pub trait KeyboardShorcut<T> {
    fn about(&self) -> String;
    fn events(&self) -> Vec<KeyEvent>;
    fn run(&self) -> Option<T>;
}

#[derive(EnumIter, Display, Debug, PartialEq, Clone)]
pub enum InteractiveCommand {
    Help,
    Reload,
    Quit,
    Stop,
}

impl InteractiveCommand {
    pub fn name(&self) -> String {
        format!("{}", self)
    }
}

impl KeyboardShorcut<DevTermCommand> for InteractiveCommand {
    fn about(&self) -> String {
        match self {
            InteractiveCommand::Help => "Print this message".into(),
            InteractiveCommand::Quit => {
                "Quit the interactive mode and open the Lenra dev terminal".into()
            }
            _ => {
                let main_command = DevCli::command();
                let command = main_command.find_subcommand(self.name().to_lowercase().as_str());
                command.unwrap().get_about().unwrap().into()
            }
        }
    }

    fn events(&self) -> Vec<KeyEvent> {
        match self {
            InteractiveCommand::Quit => vec![
                KeyEvent(KeyCode::Char('q'), Modifiers::NONE),
                KeyEvent(KeyCode::Char('C'), Modifiers::CTRL),
            ],
            _ => {
                let name = format!("{}", self);
                vec![KeyEvent::new(
                    name.to_lowercase().chars().next().unwrap(),
                    Modifiers::NONE,
                )]
            }
        }
    }

    fn run(&self) -> Option<DevTermCommand> {
        match self {
            InteractiveCommand::Help => {
                display_help();
                Some(DevTermCommand::Continue)
            }
            InteractiveCommand::Reload => Some(DevTermCommand::Reload),
            InteractiveCommand::Quit => None,
            InteractiveCommand::Stop => Some(DevTermCommand::Stop),
        }
    }
}

pub async fn listen_interactive_command() -> Result<Option<DevTermCommand>> {
    let command: Arc<Mutex<Option<DevTermCommand>>> = Arc::new(Mutex::new(None));
    let mut listener = KeyboardListener::new()?;
    InteractiveCommand::iter().for_each(|cmd| {
        cmd.events().iter().for_each(|&event| {
            let cmd = cmd.clone();
            let local_command = command.clone();
            let f = move || {
                let mut c = local_command.lock().unwrap();
                *c = cmd.run();
                debug!("{}", cmd.name());
                Some(Cmd::AcceptLine)
            };
            listener.add_listener(event, f);
        });
    });
    listener.add_listener(ENTER_EVENT, || {
        println!();
        Some(Cmd::Replace(Movement::WholeBuffer, Some("".into())))
    });
    listener.listen().await?;
    let mutex = command.lock().unwrap();
    let command = mutex.deref();
    Ok(command.clone())
}

fn display_help() {
    let mut vector = Vec::new();
    vector.extend(InteractiveCommand::iter().map(|cmd| {
        let mut shortcuts = Vec::new();
        shortcuts.extend(cmd.events().iter().map(|&e| keyevent_to_string(e)));
        format!(
            "    {:8}  {:15}  {}",
            cmd.name().color(Color::Green),
            shortcuts.join(", ").color(Color::Blue),
            cmd.about()
        )
    }));
    println!(
        "\n{} ({}  {}  {})\n{}\n",
        "SHORTCUTS:".color(Color::Yellow),
        "Command".color(Color::Green),
        "Key(s)".color(Color::Blue),
        "Description",
        vector.join("\n")
    )
}
