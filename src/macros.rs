//! This module defines macros for actor definition and invocation.
//!
//!
//![define_actor!](crate::define_actor)
//!
//![send!](crate::send)

///This macro defines an actor in the system. It takes a literal string as actor name
///and an implmentation of [Producer](crate::common::actor::Producer) that is called to
///return an [Actor](crate::common::actor::Actor). The actor becomes active as soon as it
///is defined and receives a startup signal.
///
///Each Producer implementation should also be tagged with a `typetag` attribute name that
///should not collide with any other name in the system.
///
///Example
///
///```
///use arrows::{Actor, Addr, Mail, Msg, Producer};
///use serde::{Deserialize, Serialize};
///
///pub struct NewActor;
///
///impl Actor for NewActor {
///    fn receive(&mut self, _incoming: Mail) -> Option<Mail> {
///        Some(Msg::from_text("Reply from new actor").into())
///    }
///}
///
///```
///
///Next we implement the [Producer](crate::Producer) trait to produce `NewActor`
///intances on demand. We also tag the Producer implementation with the typetag
///"new_actor_producer".
///
///```
///#[derive(Debug, Serialize, Deserialize, Default)]
///struct NewProducer;
///
///#[typetag::serde(name = "new_actor_producer")]
///impl Producer for NewProducer {
///    fn produce(&mut self) -> Box<dyn Actor> {
///        Box::new(NewActor)
///    }
///}
///
///````
///At this point - we have our `Actor` and `Producer` implementations ready. we can use
///the `define_actor` macro to register the producer into the system. The producer
///would be persisted into the system, the actor will be activated and would receive a post
///start signal. Same prodcer instance would be called to create instances of the actor at
///system restart/actor activation points at different times in the actor's life-cycle.
///
///```
///use arrows::define_actor;
///
///let actor_producer = NewProducer::default();
///define_actor!("new_actor", actor_producer);
///
///```
///At this point - the actor would be running. We can send messages to the actor which it will
///receive and sending out messages.
///
///```
///use arrows::send;
///
///let m1 = Msg::from_text("Message to new_actor");
///let m2 = Msg::from_text("Message to new_actor");
///let m3 = Msg::from_text("Message to new_actor");
///send!("new_actor", (m1, m2, m3));
///
///```
///
///We can use same producer definition to create multiple instances of the actor.
///
///```
///let producer2 = NewProducer::default();
///let producer3 = NewProducer::default();
///define_actor!("another_actor", producer2);
///define_actor!("yet_another_actor", producer3);
///
///```
///
///
#[macro_export]
macro_rules! define_actor {
    ($actor_name:literal, $actor_producer:path) => {{
        let addr = $crate::Addr::new($actor_name);
        let _res = $crate::catalog::define_actor(addr, $actor_producer);
    }};
    ($actor_addr:expr, $actor_producer:path) => {{
        let actor_addr: $crate::Addr = $actor_addr;
        let _res = $crate::catalog::define_actor(actor_addr, $actor_producer);
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
    use crate::{Actor, Mail, Msg, Producer};
    use serde::{Deserialize, Serialize};

    pub struct TestActor;

    impl Actor for TestActor {
        fn receive(&mut self, _incoming: Mail) -> std::option::Option<Mail> {
            Some(Msg::from_text("Reply from test actor").into())
        }
    }

    #[derive(Debug, Serialize, Deserialize, Default)]
    struct TestActorProducer;

    #[typetag::serde(name = "test_actor_producer")]
    impl Producer for TestActorProducer {
        fn produce(&mut self) -> Box<dyn Actor> {
            Box::new(TestActor)
        }
    }
    #[test]
    fn define_test_actor_test() {
        let producer = TestActorProducer::default();
        define_actor!("test_actor", producer);
        send!("test_actor", Msg::from_text("Test message"));
    }
}
