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
        let (sender, receiver) = channel();
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
            let directed_events = Self::perist_buffered(self.buffer.flush());
            let (inbox, outbox): (Vec<(i64, _)>, Vec<(i64, _)>) =
                directed_events.iter().partition(|e| e.1 == true);
            let inbox = inbox.iter().map(|e| e.0);
            let outbox = outbox.iter().map(|e| e.0);
            /***for event in directed_events {
              self.sender.send(event).expect("Send directed event");
            }***/
        }
    }
    //Persists the events to db
    pub(crate) fn perist_buffered(events: Vec<DBEvent>) -> Vec<(i64, bool)> {
        let directed_events = Context::instance()
            .store
            .persist_events(events.into_iter())
            .expect("Events persisted");
        println!("Clearing buffer. Directed events = {:?}", directed_events);
        directed_events
    }

    pub(crate) fn load_messages(rowids: Vec<i64>) -> Vec<Msg> {
        Context::instance()
            .store
            .from_box(rowids, true)
            .expect("Messages")
    }

    pub(crate) fn route_past_events(&mut self) {
        let ins = Context::instance()
            .store
            .read_past_events(true)
            .expect("Past inbounds");
        let outs = Context::instance()
            .store
            .read_past_events(false)
            .expect("Past outbounds");
        let ins = Self::load_messages(ins);
        let outs = Self::load_messages(outs);
        println!(
            "Routing past events. Ins = {:?} and outs = {:?}",
            ins.len(),
            outs.len()
        );
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
