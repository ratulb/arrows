use crate::{Mail, Msg};
use serde::{Deserialize, Serialize};
use std::any::{self, Any};
use std::fmt::{self, Debug, Formatter};

pub trait Actor: Any + Send + Sync {
    //! # Actor
    //!
    //!`Actor` - the trait for an actor implementation in the system. The only required method
    //!to implement is `receive` - which keeps receiving messages(`Mail`) and optionally
    //!supposed to return an outgoing message(s) which may be addressed to any local or remote
    //!actor(s). Outgoing `Mail` can contain multiple messages directed to different actors.
    //!Messages directed to an actor will always be delivered in the order that they were
    //!ingested into the actor system. Out of sequence messages will not be delivered until
    //!all previous messages have been consumed by the actor. Message might get delivered more
    //!than once because of actor failing to process message.
    //!
    //!Upon restart - actor would start receiving messages - where it left off from.

    /**
     * The required method that needs to be implemented as part of `Actor` implementation.
     * Called to handle incoming messages. The incoming message should not be returned as it
     * is - doing so would lead to re-delivery of the message back again. The 'to' `Addr`
     * of the recipient should be set in the outgoing payload.
     *
     * Incoming mail will have only a single message `Trade(Msg)` variant of `Mail` enum.
     **/

    fn receive(&mut self, _mail: Mail) -> Option<Mail> {
        Some(Mail::Blank)
    }
    /**
     * Name of the type implementing the `Actor` trait
     **/
    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }
    /**
     * The startup signal that the actor receives upon definition - when actor is defined via
     * `define_actor!` macro or on restoration from the backing store.
     **/

    fn post_start(&mut self, _mail: Mail) -> Option<Mail> {
        Some(Msg::from_text("Starting up", "from_this_actor", "to_another_actor").into())
    }

    /**
     * Pre-shutdown signal that the actor will receive due to normal shutdown or  actor
     * eviction due to actor panicking while processing message. An actor is allowed to
     * panic a set number of times(currently 3) before eviction. Actor might panic due to
     * internal(faulty logic, index bounds exception) or external reasons like message
     * getting corrupted in transit.
     **/
    fn pre_shutdown(&mut self, _mail: Mail) -> Option<Mail> {
        Some(Msg::from_text("Shutdown", "from_this_actor", "to_another_actor").into())
    }
}

impl Debug for dyn Actor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let debug_msg = format!("Actor impl({:?})", self.type_name());
        write!(f, "{}", debug_msg)
    }
}
/// # Producer
///Implementation of an `Actor` trait involves two steps. First, implementation of the
///`Actor` trait itself. Once trait implentation is done - first step is complete. Making
///Actor instances available to system is job of the `Producer`. A `Producer` trait
///implemenation(each `Producer` implementation with an unique `typetag` name) is
///responsible for creating `Actor` instances at runtime. The `Producer` trait has the
///`produce` method - which gets invoked to hand out `Actor` instances during actor
///registration and restoration. So, the second step of `Actor` implemenation is to
///implement the `Producer` trait. While registering an `Actor` to the system - the
///`define_actor!` macro takes a serde serializable(<https://github.com/serde-rs/serde>)
///`Producer` instance and a name for the actor(string - which could be anything - a
///name for the actor). The `Producer` implemenation gets serialized and stored in the
///backing store.

#[typetag::serde]
pub trait Producer {
    ///The method to be implemented to create `Actor` instances. The implementing type
    ///should be tagged with a non-collinding `typetag` name in the format
    ///#[typetag::serde(name = "an_actor_producer")]
    ///
    ///<https://github.com/dtolnay/typetag>
    ///

    fn produce(&mut self) -> Box<dyn Actor>;

    ///A method to rebuild a `Producer` implementation. Used internally by the system to
    ///generate `Producer`s on demand from the backing store.  
    fn from_string(&self, content: String) -> std::io::Result<Box<dyn Producer>> {
        let producer: Box<dyn Producer> = serde_json::from_str(&content)?;
        Ok(producer)
    }
}

//Sample actor and actor producer
pub struct ExampleActor;
impl Actor for ExampleActor {
    fn receive(&mut self, incoming: Mail) -> std::option::Option<Mail> {
        if !Mail::is_blank(&incoming) {
            println!("Actor received {}", incoming.message());
            let reply = Msg::from_text("Some text", "from_me", "to_sender");
            return Some(reply.into());
        }
        None
    }
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExampleActorProducer;
#[typetag::serde(name = "example_actor_producer")]
impl Producer for ExampleActorProducer {
    fn produce(&mut self) -> Box<dyn Actor> {
        Box::new(ExampleActor)
    }
}
////////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct ProducerDeserializer;
//ProducerDeserializer is used internally to create actor producers from their serialized
//state.

#[typetag::serde(name = "producer_deserializer")]
impl Producer for ProducerDeserializer {
    fn produce(&mut self) -> Box<dyn Actor> {
        panic!("Should not be called on ProducerDeserializer");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{define_actor, send, Addr};
    use serde::{Deserialize, Serialize};

    #[test]
    fn example_actor() {
        let producer = ExampleActorProducer;
        //Define an actor
        let _rs = define_actor!("example_actor1", producer);
        //Another actor instance with same behaviour
        let _rs = define_actor!(Addr::new("example_actor2"), ExampleActorProducer);

        let m = Msg::default();
        //Send out messages
        send!("example_actor1", m);
        send!(Addr::new("example_actor2"), Msg::default());
    }

    #[test]
    fn create_actor_test1() {
        #[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
        struct MyActor {
            name: String,
        }
        impl MyActor {
            fn new() -> Self {
                Self {
                    name: String::from("An actor"),
                }
            }
        }
        //A demo actor implementation which responds by blank replies.
        //message(_msg)
        impl Actor for MyActor {
            fn receive(&mut self, _msg: Mail) -> Option<Mail> {
                Some(Mail::Blank)
            }
        }
        let mut my_actor = MyActor::new();
        assert!(my_actor.receive(Mail::Blank).is_some());
    }

    #[test]
    fn actor_producer_test_1() {
        struct MyActor;

        impl Actor for MyActor {
            fn receive(&mut self, incoming: Mail) -> std::option::Option<Mail> {
                println!("My Actor received {}", incoming.message());
                None
            }
        }

        #[derive(Clone, Debug, Serialize, Deserialize, Default)]
        struct MyProducer;

        //Tag the impl with distinguishable name - actor producer's name should not
        //collide with existing producers' names
        #[typetag::serde(name = "my_actor_producer")]
        impl Producer for MyProducer {
            fn produce(&mut self) -> Box<dyn Actor> {
                Box::new(MyActor)
            }
        }

        let producer = MyProducer::default();
        let _rs = define_actor!("myactor", producer);
        send!("myactor", Msg::default());
    }
}
