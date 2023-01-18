use log::{debug, warn};
use rustyline::{
    Cmd, ConditionalEventHandler, Editor, Event, EventContext, EventHandler, Helper, KeyEvent,
    RepeatCount, Modifiers,
};

use crate::errors::{Error, Result};

#[derive(Debug)]
struct KeyEventHandler<F>
where
    F: Fn() -> Option<Cmd> + Send + Sync + 'static,
{
    event: KeyEvent,
    listener: F,
}

pub trait KeyEventListener<F>
where
    F: Fn() -> Option<Cmd> + Send + Sync + 'static,
{
    fn add_listener(&mut self, event: KeyEvent, listener: F) -> &mut Self;
}

impl<F, H> KeyEventListener<F> for Editor<H>
where
    F: Fn() -> Option<Cmd> + Send + Sync + 'static,
    H: Helper,
{
    fn add_listener(&mut self, event: KeyEvent, listener: F) -> &mut Self {
        let normalized_event = KeyEvent::normalize(event);
        self.bind_sequence(
            normalized_event.clone(),
            EventHandler::Conditional(Box::new(KeyEventHandler {
                event: normalized_event,
                listener: listener,
            })),
        );
        self
    }
}

impl<F> ConditionalEventHandler for KeyEventHandler<F>
where
    F: Fn() -> Option<Cmd> + Send + Sync,
{
    fn handle(&self, evt: &Event, _: RepeatCount, _: bool, _: &EventContext) -> Option<Cmd> {
        debug!("KeyEventHandler: {:?}", evt);
        if let Some(k) = evt.get(0) {
            let key = KeyEvent::normalize(*k);
            debug!("KeyEventHandler: {:?}", key);
            if key == self.event {
                return (self.listener)();
            }
        } else {
            warn!("KeyEventHandler without key");
        }
        Some(Cmd::Insert(0, "".into()))
    }
}

pub struct KeyboardListener {
    editor: Editor<()>,
}

impl<F> KeyEventListener<F> for KeyboardListener
where
    F: Fn() -> Option<Cmd> + Send + Sync + 'static,
{
    fn add_listener(&mut self, event: KeyEvent, listener: F) -> &mut Self {
        self.editor.add_listener(event, listener);
        self
    }
}

impl KeyboardListener {
    pub async fn listen(mut self) -> Result<()> {
        tokio::task::spawn_blocking(move || self.editor.readline("").map_err(Error::from))
            .await
            .map_err(Error::from)?
            .ok();
        Ok(())
    }

    pub fn new() -> Result<Self> {
        Ok(KeyboardListener {
            editor: Editor::new()?,
        })
    }
}

pub fn keyevent_to_string(event: KeyEvent) -> String{
    let mut parts = Vec::new();
    if event.1 & Modifiers::CTRL == Modifiers::CTRL {
        parts.push("Ctrl".to_string());
    }
    if event.1 & Modifiers::ALT == Modifiers::ALT {
        parts.push("Alt".to_string());
    }
    if event.1 & Modifiers::SHIFT == Modifiers::SHIFT {
        parts.push("Shift".to_string());
    }
    parts.push(match event.0 {
        rustyline::KeyCode::UnknownEscSeq => todo!(),
        rustyline::KeyCode::Backspace => todo!(),
        rustyline::KeyCode::BackTab => todo!(),
        rustyline::KeyCode::BracketedPasteStart => todo!(),
        rustyline::KeyCode::BracketedPasteEnd => todo!(),
        rustyline::KeyCode::Char(c) => c.to_string().to_uppercase(),
        rustyline::KeyCode::Delete => todo!(),
        rustyline::KeyCode::Down => todo!(),
        rustyline::KeyCode::End => todo!(),
        rustyline::KeyCode::Enter => "Enter".to_string(),
        rustyline::KeyCode::Esc => "Esc".to_string(),
        rustyline::KeyCode::F(_) => todo!(),
        rustyline::KeyCode::Home => todo!(),
        rustyline::KeyCode::Insert => todo!(),
        rustyline::KeyCode::Left => todo!(),
        rustyline::KeyCode::Null => todo!(),
        rustyline::KeyCode::PageDown => todo!(),
        rustyline::KeyCode::PageUp => todo!(),
        rustyline::KeyCode::Right => todo!(),
        rustyline::KeyCode::Tab => todo!(),
        rustyline::KeyCode::Up => todo!(),
        _ => todo!(),
    });
    parts.join(" + ")
}
