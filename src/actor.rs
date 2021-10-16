use crate::{to_file, type_of, Address, MailBox, Message};
use serde::Deserialize;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Write};

pub struct Ractor<T: Serialize, R: Serialize> {
    addr: Address,
    mailbox: Option<MailBox>,
    invokable: Box<dyn Fn(Message<T>) -> Option<Message<R>>>,
}

impl<T: Serialize, R: Serialize> Ractor<T, R> {
    //Create an actor passing a Message<T> -> Message<R> closure
    pub fn new(name: &str, invokable: Box<dyn Fn(Message<T>) -> Option<Message<R>>>) -> Self {
        Self {
            addr: Address::new(name),
            mailbox: None,
            invokable,
        }
    }
    pub async fn receive(&mut self, msg: Message<T>) -> Option<Message<R>> {
        println!("Actor received message");
        (self.invokable)(msg)
    }
}

pub trait Actor {
    fn receive<'a, R, T>(message: Message<T>) -> Option<Message<R>>
    where
        T: Clone + std::fmt::Debug + Serialize + Deserialize<'a>,
        R: Clone + std::fmt::Debug + Serialize + Deserialize<'a>,
    {
        //Default implementation - override as needed
        println!("Received message: {:#?}", message);
        None
    }
}
