use common::Msg;
use common::{Actor, ActorBuilder};
use registry::register;
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
    let _reg_result = register(1000, NewActorBuilder::default());
}
