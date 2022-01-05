use crate::constants::{BUFFER_MAX_SIZE, EVENT_MAX_AGE};
use crate::events::DBEvent;
use crate::registry::Context;
use crate::Msg;
use std::mem;
use std::sync::mpsc::{channel, Sender};
use std::time::{Duration, Instant};

pub(crate) struct InboxRouter;
pub(crate) struct OutboxRouter;

pub(crate) struct Router {
    buffer: EventBuffer,
    sender: Sender<(Msg, bool)>,
}

impl Router {
    pub(crate) fn new() -> Self {
        let (sender, _receiver) = channel();
        let mut this = Self {
            buffer: EventBuffer::new(),
            sender,
        };
        Self::route_past_events(&mut this);
        this
    }
    pub(crate) fn route(&mut self, event: DBEvent) {
        self.buffer.add(event);
        if self.buffer.should_flush() {
            let _persisted_events = Self::perist_buffered(self.buffer.flush());
            /***for event in directed_events {
              self.sender.send(event).expect("Send directed event");
            }***/
        }
    }
    //Persists the events to db
    pub(crate) fn perist_buffered(events: Vec<DBEvent>) -> Vec<i64> {
        let persisted_events = Context::instance()
            .store
            .persist_events(events.into_iter())
            .expect("Events persisted");
        println!("Clearing buffer. Persisted events = {:?}", persisted_events);
        persisted_events
    }

    pub(crate) fn load_messages(rowids: Vec<i64>) -> Vec<(Msg, i64)> {
        Context::instance()
            .store
            .from_messages(rowids)
            .expect("Messages")
    }

    pub(crate) fn route_past_events(&mut self) {
        let events = Context::instance()
            .store
            .read_past_events()
            .expect("Past events");
        let events = Self::load_messages(events);
        println!("Routing past events. Events = {:?}", events.len());
    }
}
/***
Message1 - Actor1

Message2 - Actor2

Message3 - Actor1

Message4 -> Actor2

Message5 -> Actor3

****************

Message6 -> Actor3

Message7 -> Actor2

Message8 - Actor1

Message9 - Actor2

Message10 - Actor1

*****************

Message11 -> Actor2

Message12 -> Actor3
***/
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
    }
    pub(crate) fn flush(&mut self) -> Vec<DBEvent> {
        mem::take(&mut self.events)
    }
}
impl Drop for EventBuffer {
    fn drop(&mut self) {
        let events = self.flush();
        if !events.is_empty() {
            Router::perist_buffered(events);
        }
    }
}
