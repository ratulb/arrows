use arrows::catalog::restore;
use arrows::define_actor;
use arrows::send;

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
    let identity = Addr::new("new_actor");
    let rs = restore(identity);
    println!("The rs = {:?}", rs);

    let builder = NewActorBuilder::default();

    let rs = define_actor!("new_actor", builder);
    println!("The reg result is = {:?}", rs);

    let builder = NewActorBuilder;
    define_actor!(Addr::new("new_actor"), builder);

    let m = Msg::default();
    send!("new_actor", m);

    send!(Addr::new("new_actor"), Msg::default());

    let msg_to_unregisterd = Msg::new_with_text("Mis-directed message", "from", "to");
    send!("Unknown actor", msg_to_unregisterd);
}
