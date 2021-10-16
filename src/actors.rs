use crate::{Actor, Address, MailBox, Message, Ractor, STORE};
use serde::Serialize;
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
        let addr = Address::new(name);
        Ractor::new(name, Box::new(invokable))
    }

    pub fn ractor_of(name: &str, ractor: impl Actor) -> Result<ActorArrow> {
        Ok(ActorArrow)
    }
}
enum ActorCatalog {
    ActorBuilder,
}

struct ActorBuilder;

impl Actor for ActorBuilder {}