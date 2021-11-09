#[macro_export]
macro_rules! register {
    ($actor_name:literal, $actor_builder:path) => {
        let identity = $crate::Addr::new($actor_name).get_id();
        $crate::registry::register_builder(identity, $actor_builder);
    };
}

#[macro_export]
macro_rules! send {
    ($actor_name:literal, $msg:expr) => {
        let msg: $crate::Msg = $msg;
        let identity = $crate::Addr::new($actor_name).get_id();
        $crate::registry::send(identity, msg);
    };
}

/***
 * 1) Fix location of arrows.db
2) Event loop
3) Send macro - done
4) cfg to check selected scheme
5) Message routing to inbox/outbox
6) Binaries for server/Client
7) Message exchange format for client and server
8) Make everything async
8) Multithreading
9) Json message out from stream
10) db trimming
11) Documentation
***/

#[cfg(test)]
mod tests {
    use crate::{Actor, ActorBuilder, Msg};
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
    #[test]
    fn macro_register_actor_test1() {
        let actor_builder = NewActorBuilder::default();
        register!("new_actor", actor_builder);
        send!("new_actor", Msg::Blank);
    }
}
