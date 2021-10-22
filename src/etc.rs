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

pub struct Ractor<'b: 'c, 'c, 'a> {
    addr: Address<'a>,
    mailbox: Option<Mailbox<'a>>,
    invokable: Box<dyn Fn(Message<'b>) -> Option<Message<'c>>>,
}

impl<'b: 'c, 'c, 'a> Ractor<'b, 'c, 'a> {
    //Create an actor passing a Message -> Message closure
    pub fn new(name: &'a str, invokable: Box<dyn Fn(Message<'b>) -> Option<Message<'c>>>) -> Self {
        Self {
            addr: Address::new(name),
            mailbox: None,
            invokable,
        }
    }
    pub async fn receive(&mut self, msg: Message<'b>) -> Option<Message<'c>> {
        println!("Actor received message");
        (self.invokable)(msg)
    }
}
