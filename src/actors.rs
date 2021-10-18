use crate::{Actor, Address, Message, Ractor};

use serde::Serialize;
use std::collections::HashMap;

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
pub(crate) enum SysAddrs {
    test,
}

pub(crate) struct SysActors {
    sys_actors: HashMap<u64, Box<dyn Actor>>,
}

impl SysActors {
    pub(crate) fn new() -> Self {
        Self {
            sys_actors: HashMap::new(),
        }
    }
}
/***
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
}***/
pub(crate) const SYS_REQUEST_VALIDATOR: &str = "sys-request-validator";
pub(crate) struct ActorInitializer;

pub(crate) struct RequestValidator<'a> {
    addr: Address<'a>,
}

impl<'a> RequestValidator<'a> {
    pub(crate) fn new() -> Self {
        dbg!("Request validator starting with assumed name of \"sys-request-validator\"");
        Self {
            addr: Address::new(SYS_REQUEST_VALIDATOR),
        }
    }
}

impl<'a> Actor for RequestValidator<'a> {
    fn receive<'b: 'c, 'c>(&mut self, incoming: &mut Message<'b>) -> Option<Message<'c>> {
        dbg!("Received validation message - allowing to proceed");
        let _incoming1 = incoming.uturn_with_text("Request validation passed");
        //Some(Message::new(None,"from", "to"))
        //Some(incoming1)
        None
    }
}
/***
impl Actor for ActorInitializer {
    fn receive<(&mut self, _msg: Message) -> Option<Message> {
        None
    }
}***/
