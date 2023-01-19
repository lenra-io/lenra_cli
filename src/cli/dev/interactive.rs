use clap::CommandFactory;
use colored::{Color, Colorize};
use rustyline::{KeyCode, KeyEvent, Modifiers};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::keyboard_event::keyevent_to_string;

use super::terminal::{DevCli, DevTermCommand};

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
