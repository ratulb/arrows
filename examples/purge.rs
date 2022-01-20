use arrows::catalog::restore;
use arrows::define_actor;
use arrows::send;

use arrows::{Actor, Addr, Mail, Msg, Producer};
use serde::{Deserialize, Serialize};

pub struct NewActor;

impl Actor for NewActor {
    fn receive(&mut self, _incoming: Mail) -> std::option::Option<Mail> {
        Some(Msg::with_text("Reply from new actor", "from", "to").into())
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct NewProducer;

#[typetag::serde(name = "new_purge_actor_producer")]
impl Producer for NewProducer {
    fn produce(&mut self) -> Box<dyn Actor> {
        Box::new(NewActor)
    }
}

fn main() {
    let identity = Addr::new("new_actor");
    let rs = restore(identity);
    println!("The rs = {:?}", rs);

    let builder = NewProducer::default();

    let rs = define_actor!("new_actor", builder);
    println!("The reg result is = {:?}", rs);

    let builder = NewProducer;
    define_actor!(Addr::new("new_actor"), builder);

    let m = Msg::default();
    send!("new_actor", m);

    send!(Addr::new("new_actor"), Msg::default());

    let msg_to_unregisterd = Msg::with_text("Mis-directed message", "from", "to");
    send!("Unknown actor", msg_to_unregisterd);
}
