use crate::{Actor, Address, Message, Ractor, type_of};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bincode::{deserialize, serialize};
use std::io::Result;
pub struct Router {}

pub struct Actors;
pub struct ActorArrow;
impl Actors {
    pub fn actor_from<
        T: Serialize,
        R: Serialize,
        F: 'static + Serialize + Fn(Message<T>) -> Option<Message<R>>,
    >(
        name: &str,
        invokable: F,
    ) -> Ractor<T, R> {
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
    fn receive<'a, R, T>(&mut self, message: Message<T>) -> Option<Message<R>>
    where
        T: Clone + std::fmt::Debug + Serialize + Deserialize<'a>,
        R: Clone + std::fmt::Debug + Serialize + Deserialize<'a>,
    {
        //Default implementation - override as needed
        println!("Received message: {:#?}", message);
         let content = message.get_content();
         let value = match content {
            Some(v) => v,
            _=> panic!("Close call"),
         };
         let value = *value as Vec<u8>;
         let decoded: Message<Complex<Inner>> = deserialize(&value[..]).unwrap();
        println!("Received message =======*******========");
    match decoded {
        Message::Custom { content, .. } => {
            if let Some(complex) = content {
                if let Complex { inner, elems } = complex {
                    println!("Inner = {:?}", inner);
                    println!("Elems {:?} ", elems);
                    type_of(&elems);
                    println!("Elems len {:?} ", elems.len());
                    println!("At position 0 {:?} ", elems[0]);
                }
            }
        }
        _ => (),
    }

    println!("============**********===============");






        let result: Option<Message<R>> = None;
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
