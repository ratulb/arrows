use crate::constants::{BUFFER_MAX_SIZE, EVENT_MAX_AGE};
use crate::events::{DBEvent, Events};
use crate::registry::Context;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

pub(crate) struct InboxRouter;
pub(crate) struct OutboxRouter;
pub(crate) struct ExternalRouter;

pub(crate) struct Router {
    receiver: Receiver<Events>,
}

pub(crate) struct EventBuffer {
    events: Vec<DBEvent>,
    oldest_receipt_instant: Option<Instant>,
}
impl EventBuffer {
    pub(crate) fn new() -> Self {
        Self {
            events: Vec::new(),
            oldest_receipt_instant: None,
        }
    }
    pub(crate) fn overflown(&self) -> bool {
        self.events.len() >= BUFFER_MAX_SIZE
    }
    pub fn oldest_matured(&self) -> bool {
        match self.oldest_receipt_instant {
            None => false,
            Some(instant) => instant.elapsed() >= Duration::new(EVENT_MAX_AGE, 0),
        }
    }
    pub(crate) fn should_route_messages(&self) -> bool {
        self.overflown() || self.oldest_matured()
    }
    pub(crate) fn add_event(&mut self, event: DBEvent) {
        self.events.push(event);
        if self.events.len() == 1 {
            self.oldest_receipt_instant = Some(Instant::now());
        }
        if self.should_route_messages() {
            let events = std::mem::take(&mut self.events);
            Self::route_messages(events);
        }
    }
    pub(crate) fn route_messages(events: Vec<DBEvent>) {
        let (ins, outs) = Context::instance()
            .store
            .persist_dbevents(events.into_iter())
            .expect("Events persisted");
        println!("Ins = {:?} and outs = {:?}", ins, outs);
    }
}
impl Drop for EventBuffer {
    fn drop(&mut self) {
        let events = std::mem::take(&mut self.events);
        if !events.is_empty() {
            Self::route_messages(events);
        }
    }
}
