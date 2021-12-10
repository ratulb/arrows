#[macro_export]
macro_rules! builder_of {
    ($actor_name:literal, $actor_builder:path) => {{
        let identity = $crate::Addr::new($actor_name).get_id();
        let res = $crate::registry::register_builder(identity, $actor_builder);
        //println!("Res result is = {:?}", res);
        res
    }};
    ($actor_addr:expr, $actor_builder:path) => {{
        let actor_addr: $crate::Addr = $actor_addr;
        let identity = actor_addr.get_id();
        let res = $crate::registry::register_builder(identity, $actor_builder);
        res
    }};
}

#[macro_export]
macro_rules! send_to {
    ($actor_name:literal, $msg:expr) => {
        let msg: $crate::Msg = $msg;
        let identity = $crate::Addr::new($actor_name).get_id();
        $crate::registry::send(identity, msg);
    };
    ($actor_addr:expr, $msg:expr) => {
        let msg: $crate::Msg = $msg;
        let actor_addr: $crate::Addr = $actor_addr;
        let identity = actor_addr.get_id();
        $crate::registry::send(identity, msg);
    };
}

/***
1) Fix location of arrows.db
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
12) Error handling
13) Macro output
14) Logging
***/

#[cfg(test)]
mod tests {
    use crate::{Actor, ActorBuilder, Addr, Mail, Msg};
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
    #[test]
    fn macro_register_actor_test1() {
        let builder = NewActorBuilder::default();
        builder_of!("new_actor", builder);

        let builder = NewActorBuilder::default();
        builder_of!(Addr::new("new_actor"), builder);

        let builder = NewActorBuilder::default();
        let addr = Addr::new("new_actor");
        builder_of!(addr, builder);

        send_to!("new_actor", Msg::default());
        send_to!(Addr::new("new_actor"), Msg::default());
        let addr = Addr::new("new_actor");
        send_to!(addr, Msg::default());
    }
}
