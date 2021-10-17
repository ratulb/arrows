use crate::{type_of, Actor, Address, Message, Ractor};
use bincode::deserialize;
use serde::{Deserialize, Serialize};

use std::io::Result;
pub struct Router {}

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

struct WrappedActor {}

struct ActorCatalog {
    //actor_impls: HashMap<String, WrappedActor>,
}

//Need to be made crate private
#[derive(Debug, Clone)]
pub struct ActorBuilder;

impl Actor for ActorBuilder {
    fn receive(&mut self, message: Message) -> Option<Message> {
        //Default implementation - override as needed
        println!("Received message: {:#?}", message);
        let content = message.get_content();
        let empty_data = vec![];
        let content = match content {
            Some(ref value) => value,
            None => &empty_data,
        };

        println!("Received message buffer length = {}", content.len());
        type_of(&content);

        let decoded: Complex<Inner> = deserialize(&content[..]).unwrap();

        println!("{:?}", decoded);
        println!("============**********===============");

        let result: Option<Message> = None;
        result
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
struct Complex<T> {
    inner: T,
    elems: Vec<Simple>,
}
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
struct Inner {
    name: String,
    children: Vec<String>,
    male: bool,
    age: u8,
}
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
struct Simple {
    e1: i32,
    e2: usize,
    e3: Option<bool>,
}
