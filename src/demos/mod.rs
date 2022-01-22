//! #Demos
//! Contains various demo actor definitions. Actors defined - should be part of the final
//! binarirs - hence this experimental module.
//!
use crate::define_actor;
use crate::{Actor, Mail, Msg, Producer};
use serde::{Deserialize, Serialize};
pub struct NewActor;
impl Actor for NewActor {
    fn receive(&mut self, _incoming: Mail) -> Option<Mail> {
        println!("The implementing type {}", self.type_name());
        Some(Msg::from_text("Reply from new actor").into())
    }
}
#[derive(Debug, Serialize, Deserialize, Default)]
struct NewProducer;

#[typetag::serde]
impl Producer for NewProducer {
    fn produce(&mut self) -> Box<dyn Actor> {
        Box::new(NewActor)
    }
}

pub fn register() {
    let actor_producer = NewProducer::default();
    define_actor!("new_actor", actor_producer);
}
