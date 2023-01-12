use log::{debug, warn};
use rustyline::{
    Cmd, ConditionalEventHandler, Editor, Event, EventContext, EventHandler, KeyEvent, RepeatCount,
};

#[derive(Debug)]
pub struct KeyEventHandler<F>
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
    fn listen(self, event: KeyEvent, listener: F) -> Self;
}

impl<F> KeyEventListener<F> for Editor<()>
where
    F: Fn() -> Option<Cmd> + Send + Sync + 'static,
{
    fn listen(mut self, event: KeyEvent, listener: F) -> Self {
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
