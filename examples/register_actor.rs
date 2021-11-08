use arrows::registry::registry::register;
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
    let reg_result = register(1000, NewActorBuilder::default());
    println!("1000 reg result: {:?}", reg_result);
    let reg_result = register(2000, NewActorBuilder::default());
    println!("2000 reg result: {:?}", reg_result);
    send(2000, Msg::Blank);
    send(2000, Msg::Blank);
    send(2000, Msg::Blank);
    send(2000, Msg::Blank);
    send(1000, Msg::Blank);
    send(1000, Msg::Blank);
    send(3000, Msg::Blank);
    send(3000, Msg::Blank);
}
