use crate::{Actor, Address, Mailbox, Message};
use serde::Serialize;
use std::io::Result;

pub struct Actors;
pub struct ActorArrow;
impl Actors {
    pub fn actor_from<F: 'static + Serialize + Fn(Message) -> Option<Message>>(
        name: &str,
        invokable: F,
    ) -> Ractor {
        let _addr = Address::new(name);
        Ractor::new(name, Box::new(invokable))
    }
    pub fn ractor_of(_name: &str, _ractor: impl Actor) -> Result<ActorArrow> {
        Ok(ActorArrow)
    }
}

pub struct Ractor {
    addr: Address,
    mailbox: Option<Mailbox>,
    invokable: Box<dyn Fn(Message) -> Option<Message>>,
}

impl  Ractor {
    //Create an actor passing a Message -> Message closure
    pub fn new(name: &str, invokable: Box<dyn Fn(Message) -> Option<Message>>) -> Self {
        Self {
            addr: Address::new(name),
            mailbox: None,
            invokable,
        }
    }
    pub async fn receive(&mut self, msg: Message) -> Option<Message> {
        println!("Actor received message");
        (self.invokable)(msg)
    }
}
