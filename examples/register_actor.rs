use arrows::builder_of;
use arrows::send_to;
use arrows::{Actor, ActorBuilder, Addr, Mail, Msg};
use serde::{Deserialize, Serialize};

pub struct NewActor;

impl Actor for NewActor {
    fn receive(&mut self, _incoming: Mail) -> std::option::Option<Mail> {
        Some(Msg::new_with_text("Reply from new actor", "from", "to").into())
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
    let builder = NewActorBuilder::default();

    let rs = builder_of!("new_actor", builder);
    println!("The reg result is = {:?}", rs);

    let builder = NewActorBuilder;
    builder_of!(Addr::new("new_actor"), builder);

    let m = Msg::default();
    send_to!("new_actor", m);

    send_to!(Addr::new("new_actor"), Msg::default());

    let msg_to_unregisterd = Msg::new_with_text("Mis-directed message", "from", "to");
    send_to!("Unknown actor", msg_to_unregisterd);
}
