use crate::{Actor, Address, Message, STORE};
use std::collections::HashMap;
use std::io;

pub(crate) const REQUEST_VALIDATOR: &str = "request-validator";

#[derive(Debug)]
pub(crate) struct SysActors {
    pub(crate) sys_actors: HashMap<u64, Box<dyn Actor>>,
}
unsafe impl Send for SysActors {}
unsafe impl Sync for SysActors {}
impl SysActors {
    pub(crate) fn new() -> Self {
        Self {
            sys_actors: HashMap::new(),
        }
    }
}

pub(crate) struct ActorInitializer;
pub(crate) struct ActorInvoker;
impl ActorInvoker {
    fn invoke(incoming: Message) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) struct RequestValidator<'a> {
    addr: Address<'a>,
}

impl<'a> RequestValidator<'a> {
    pub(crate) fn new() -> Self {
        dbg!(
            "Request validator starting with assumed name of \"{}\"",
            REQUEST_VALIDATOR
        );
        Self {
            addr: Address::new(REQUEST_VALIDATOR),
        }
    }
}

impl<'a> Actor for RequestValidator<'a> {
    fn receive<'i: 'o, 'o>(&mut self, incoming: &mut Message<'i>) -> Option<Message<'o>> {
        dbg!("Received validation message - allowing to proceed");
        incoming.uturn_with_text("Request validation passed");
        let outgoing = std::mem::replace(incoming, Message::Blank);
        Some(outgoing)
    }
}
impl Actor for ActorInitializer {
    fn receive<'i: 'o, 'o>(&mut self, incoming: &mut Message<'i>) -> Option<Message<'o>> {
        None
    }
}
