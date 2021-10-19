use crate::{Actor, Address, MailBox, Message};
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

pub struct Ractor<'a, 'b> {
    addr: Address<'a>,
    mailbox: Option<MailBox>,
    invokable: Box<dyn Fn(Message<'b>) -> Option<Message<'b>>>,
}

impl<'a, 'b> Ractor<'a, 'b> {
    //Create an actor passing a Message -> Message closure
    pub fn new(name: &'a str, invokable: Box<dyn Fn(Message<'b>) -> Option<Message<'b>>>) -> Self {
        Self {
            addr: Address::new(name),
            mailbox: None,
            invokable,
        }
    }
    pub async fn receive(&mut self, msg: Message<'b>) -> Option<Message<'b>> {
        println!("Actor received message");
        (self.invokable)(msg)
    }
}
