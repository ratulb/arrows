use crate::{Address, MailBox, Message};

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

pub trait Actor {
    fn receive<'a, 'b>(&mut self, message: Message<'a>) -> Option<Message<'b>> {
        //Default implementation - override as needed
        println!("Received message: {:#?}", message);
        None
    }
}
