use crate::{Address, MailBox, Message};

pub struct Ractor {
    addr: Address,
    mailbox: Option<MailBox>,
    invokable: Box<dyn Fn(Message) -> Option<Message>>,
}

impl Ractor {
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

pub trait Actor {
    fn receive(&mut self, message: Message) -> Option<Message> {
        //Default implementation - override as needed
        println!("Received message: {:#?}", message);
        None
    }
}
