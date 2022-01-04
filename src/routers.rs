use crate::constants::{BUFFER_MAX_SIZE, EVENT_MAX_AGE};
use crate::events::DBEvent;
use crate::registry::Context;
use std::{
    mem,
    time::{Duration, Instant},
};

pub(crate) struct InboxRouter;
pub(crate) struct OutboxRouter;

pub(crate) struct Router {
    buffer: EventBuffer,
}

impl Router {
    pub(crate) fn new() -> Self {
        Self::route_past_events();
        Self {
            buffer: EventBuffer::new(),
        }
    }
    pub(crate) fn route(&mut self, event: DBEvent) {
        self.buffer.add(event);
    }
    pub(crate) fn route_all(events: Vec<DBEvent>) {
        let (ins, outs) = Context::instance()
            .store
            .persist_events(events.into_iter())
            .expect("Events persisted");
        println!("Clearing buffer. Ins = {:?} and outs = {:?}", ins, outs);
    }

    pub(crate) fn route_past_events() {
        let ins = Context::instance()
            .store
            .read_past_events(true)
            .expect("Past inbounds");
        let outs = Context::instance()
            .store
            .read_past_events(false)
            .expect("Past outbounds");
        let ins = Context::instance().store.from_inbox(ins).expect("Incoming");
        let outs = Context::instance()
            .store
            .from_inbox(outs)
            .expect("Outgoing");
        println!(
            "Routing past events. Ins = {:?} and outs = {:?}",
            ins.len(),
            outs.len()
        );
    }
}

pub(crate) struct EventBuffer {
    events: Vec<DBEvent>,
    first_event_receipt_at: Option<Instant>,
}
impl EventBuffer {
    pub(crate) fn new() -> Self {
        Self {
            events: Vec::new(),
            first_event_receipt_at: None,
        }
    }
    pub(crate) fn overflown(&self) -> bool {
        self.events.len() >= BUFFER_MAX_SIZE
    }
    pub fn has_matured(&self) -> bool {
        match self.first_event_receipt_at {
            None => false,
            Some(instant) => instant.elapsed() >= Duration::new(EVENT_MAX_AGE, 0),
        }
    }
    pub(crate) fn should_flush(&self) -> bool {
        self.overflown() || self.has_matured()
    }
    pub(crate) fn add(&mut self, event: DBEvent) {
        println!("Buffering event = {:?}", event);
        self.events.push(event);
        if self.events.len() == 1 {
            self.first_event_receipt_at = Some(Instant::now());
        }
        if self.should_flush() {
            let events = mem::take(&mut self.events);
            Router::route_all(events);
        }
    }
}
impl Drop for EventBuffer {
    fn drop(&mut self) {
        let events = mem::take(&mut self.events);
        if !events.is_empty() {
            Router::route_all(events);
        }
    }
}
