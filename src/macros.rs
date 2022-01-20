//! This module defines macros for actor definition and invocation.
//!

//!This macro defines an actor in the system. It takes a literal string as actor name &
//!an implmentation of `Producer` that is called to return an `Actor`. The actor becomes
//!active as soon as it is defined and receives a startup message.
//!

///Define an actor in the system - with a literal string identifier or
///[Addr](crate::common::addr::Addr) for the actor and an implementation
///of [Producer](crate::common::actor::Producer) that is able to return a specific
///[Actor](crate::common::actor::Actor) instance on demand. The `produce` method of
///the supplied `Producer` implementation is called at time of definition,
///restoration/restart of the actor.
///
///Each Producer implementation should also be tagged with a `typetag` attribute name that
///should not collide with any other name in the system.
///
///

#[macro_export]
macro_rules! define_actor {
    ($actor_name:literal, $actor_producer:path) => {{
        let identity = $crate::Addr::new($actor_name).get_id();
        let addr = $crate::Addr::new($actor_name);
        let _res = $crate::catalog::define_actor(identity, addr, $actor_producer);
    }};
    ($actor_addr:expr, $actor_producer:path) => {{
        let actor_addr: $crate::Addr = $actor_addr;
        let identity = actor_addr.get_id();
        let _res = $crate::catalog::define_actor(identity, actor_addr, $actor_producer);
    }};
}
///Sends one or more messages to one or more actors defined in the system.
///This function is responsible for gathering and dispatching messages received from the
///macro invocation of `send!`. Multiple messages can be grouped for one more actors in
///one `send!` macro invocation as shown below:
///
///Example
///
///```
///use arrows::send;
///use arrows::Msg;
///
///let m1 = Msg::with_text("Message to actor1");
///let m2 = Msg::with_text("Message to actor1");
///let m3 = Msg::with_text("Message to actor2");
///let m4 = Msg::with_text("Message to actor1");
///let m5 = Msg::with_text("Message to actor1");
///send!("actor1", (m1, m2), "actor2", (m3), "actor1", (m4, m5));
///```
///Grouping within braces is not necessary while sending only to one actor:
///
///```
///let m6 = Msg::with_text("Message to actor3")
///let m7 = Msg::with_text("Message to actor3")
///send!("actor3",m6,m7);
///
///```
///Actors identified with string literal such as 'actor3' is assumed to be running in the
///local system(they would be resurrected - if they are not - on message send).
///
///Actors running in remote systems - need to identified by the `Addr` construct:
///
///```
///use arrows::Addr;
///
///let remote_addr1 = Addr::remote("actor1", "10.10.10.10:7171");
///let remote_addr2 = Addr::remote("actor2", "11.11.11.11:8181");
///
///let m1 = Msg::with_text("Message to remote actor1");
///let m2 = Msg::with_text("Message to remote actor1");
///let m3 = Msg::with_text("Message to remote actor2");
///let m4 = Msg::with_text("Message to remote actor2");
///
///send!(remote_addr1, (m1,m2), remote_addr2, (m3,m4));
///
///```
#[macro_export]
macro_rules! send {
    ($($actor_name:literal, ($($msg:expr),*)),*)  => {
        $crate::send!(@DELEGATE; $($crate::send!(@TO_ADDR; $actor_name), ($($msg),*)),*);
    };

    ($($actor_name:literal, $($msg:expr),*),*) => {
        $crate::send!(@DELEGATE; $($crate::send!(@TO_ADDR; $actor_name), ($($msg),*)),*);
    };

    ($($addr:expr, ($($msg:expr),*)),*)  => {
        $crate::send!(@DELEGATE; $($addr, ($($msg),*)),*);
    };

    ($($addr:expr, $($msg:expr),*),*) => {
        $crate::send!(@DELEGATE; $($addr, ($($msg),*)),*);
    };
    (@DELEGATE; $($addr:expr, ($($msg:expr),*)),*) => {{
        let mut actor_msgs = std::collections::HashMap::new();
            $(
                let addr: $crate::Addr = $addr;
                let size = $crate::send![@SIZE; $($msg),*];
                let msgs = actor_msgs.entry(&addr)
                    .or_insert(Vec::with_capacity(size));
                    $(
                        let msg: $crate::Msg = $msg;
                        msgs.push(msg);
                    )*
            )*
            $crate::recv(actor_msgs);
    } };

    (@SIZE; $($msg:expr),*) => {
        <[()]>::len(&[$($crate::send![@SUB; $msg]),*])
    };

    (@SUB; $_msg:expr) => {()};

    (@TO_ADDR; $actor_name:literal) => {
        $crate::Addr::new($actor_name)
    };
}

#[cfg(test)]
mod tests {
    use crate::{Actor, Addr, Mail, Msg, Producer};
    use serde::{Deserialize, Serialize};

    pub struct NewActor;

    impl Actor for NewActor {
        fn receive(&mut self, _incoming: Mail) -> std::option::Option<Mail> {
            Some(Msg::from_text("Reply from new actor", "from", "to").into())
        }
    }

    #[derive(Debug, Serialize, Deserialize, Default)]
    struct NewProducer;

    #[typetag::serde(name = "actor_producer_new")]
    impl Producer for NewProducer {
        fn produce(&mut self) -> Box<dyn Actor> {
            Box::new(NewActor)
        }
    }
    #[test]
    fn macro_register_actor_test1() {
        let builder = NewProducer::default();
        define_actor!("new_actor", builder);

        let builder = NewProducer::default();
        define_actor!(Addr::new("new_actor"), builder);

        let builder = NewProducer::default();
        let addr = Addr::new("new_actor");
        define_actor!(addr, builder);

        send!("new_actor", Msg::default());
        send!(Addr::new("new_actor"), Msg::default());
        let addr = Addr::new("new_actor");
        send!(addr, Msg::default());
    }
}
