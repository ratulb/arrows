use arrows::register;
use arrows::send;
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
    register!("1000", actor_builer);
    let actor_builer = NewActorBuilder::default();
    register!("2000", actor_builer);
    let m = Msg::Blank;
    send!("2000", m);
    send!("2000", Msg::Blank);
    let not_blank = Msg::new_with_text("Reply from new actor", "from", "to");
    send!("3000", not_blank);
}
