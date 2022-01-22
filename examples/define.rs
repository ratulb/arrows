use arrows::define_actor;
use arrows::send;
use arrows::{Actor, Mail, Msg, Producer};
use serde::{Deserialize, Serialize};

pub struct NewActor;

impl Actor for NewActor {
    fn receive(&mut self, _incoming: Mail) -> Option<Mail> {
        println!("I am the actor {}", self.type_name());
        Some(Msg::from_text("Reply from new actor").into())
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct NewProducer;
#[derive(Debug, Serialize, Deserialize, Default)]
struct NewProducer2;

//#[typetag::serde(name = "new_actor_producer")]
#[typetag::serde]
impl Producer for NewProducer {
    fn produce(&mut self) -> Box<dyn Actor> {
        Box::new(NewActor)
    }
}

#[typetag::serde]
impl Producer for NewProducer2 {
    fn produce(&mut self) -> Box<dyn Actor> {
        Box::new(NewActor)
    }
}

fn main() {
    let actor_producer = NewProducer::default();
    //define the actor
    define_actor!("new_actor", actor_producer);

    let m1 = Msg::from_text("Message to new_actor");
    let m2 = Msg::from_text("Message to new_actor");
    let m3 = Msg::from_text("Message to new_actor");
    //Send messages
    send!("new_actor", (m1, m2, m3));

    let producer2 = NewProducer::default();
    let producer3 = NewProducer::default();
    //Multiple instances of the same actor definition
    define_actor!("another_actor", producer2);
    define_actor!("yet_another_actor", producer3);
}
