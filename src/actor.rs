use crate::{to_file, type_of, Address, MailBox, Message, AddressMode};
use serde::Deserialize;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};

pub struct Actor<T: Serialize, R: Serialize> {
    addr: Address,
    mailbox: Option<MailBox>,
    invokable: Box<dyn Fn(Message<T>) -> Option<Message<R>>>,
}

impl<T: Serialize, R: Serialize> Actor<T, R> {
    //Create an actor passing a Message<T> -> Message<R> closure
    pub fn new(name: &str, invokable: Box<dyn Fn(Message<T>) -> Option<Message<R>>>) -> Self {
        Self {
            addr: Address::new(name),
            mailbox: None,
            invokable: invokable,
        }
    }
    pub async fn receive(&mut self, msg: Message<T>) -> Option<Message<R>> {
        println!("Actor received message");
        (self.invokable)(msg)
    }
}

pub trait Ractor {
    fn receive<R, T>(message: Message<T>) -> Option<Message<R>>
    where
        T: Serialize + std::fmt::Debug,
        R: Serialize,
    {
        //Default implementation - override as needed
        println!("Received message: {:#?}", message);
        None
    }
}
