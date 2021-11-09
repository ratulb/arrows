use arrows::register_actor;
use arrows::registry::registry::send;
use arrows::{Actor, ActorBuilder, Msg};
use serde::{Deserialize, Serialize};

pub struct NewActor;

impl Actor for NewActor {
    fn receive(&mut self, _incoming: Msg) -> std::option::Option<Msg> {
        Some(Msg::new_with_text("Reply from new actor", "from", "to"))
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct NewActorBuilder;

#[typetag::serde(name = "new_actor_builder")]
impl ActorBuilder for NewActorBuilder {
    fn build(&mut self) -> Box<dyn Actor> {
        Box::new(NewActor)
    }
}

fn main() {
    let actor_builer = NewActorBuilder::default();
    register_actor!("1000", actor_builer);
    //println!("1000 reg result: {:?}", reg_result);
    let actor_builer = NewActorBuilder::default();
    register_actor!("2000", actor_builer);
    //println!("2000 reg result: {:?}", reg_result);
    send(2000, Msg::Blank);
    send(3000, Msg::Blank);
}
