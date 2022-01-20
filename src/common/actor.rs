use crate::{Mail, Msg};
use serde::{Deserialize, Serialize};
use std::any::{self, Any};
use std::fmt::{self, Debug, Formatter};

pub trait Actor: Any + Send + Sync {
    //! # Actor
    //!
    //!`Actor` - the trait for an actor implementation in the system. The only required method
    //!to implement is `receive` - which keeps receiving messages[Mail](crate::common::msg::Ma    //!il and optionally supposed to return an outgoing message(s) which may be addressed
    //!to any local or remote actor(s). Outgoing `Mail` can contain multiple messages
    //!directed to different actors. Messages directed to an actor will always be delivered
    //!in the order that they were ingested into the actor system. Out of sequence
    //!messages will not be delivered until all previous messages have been consumed by
    //!the actor. Message might get delivered more than once because of actor failing to
    //!process message.
    //!
    //!Upon restart - actor would start receiving messages - where it left off from.

    ///The required method that needs to be implemented as part of `Actor` implementation.
    ///Called to handle incoming messages. The incoming message should not be returned as it
    ///is - doing so would lead to re-delivery of the message back again. The 'to' `Addr`
    ///of the recipient should be set emptied out in the outgoing payload.
    ///
    ///Incoming payload will be of type [Trade(Msg)](crate::common::mail::Mail).
    ///

    fn receive(&mut self, mail: Mail) -> Option<Mail>;

    ///
    ///Name of the type implementing the `Actor` trait
    ///
    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }
    ///
    ///The startup signal that the actor receives upon definition - when actor is defined via
    ///[`define_actor!`](crate::define_actor) macro or on restoration from the backing store.
    ///

    fn post_start(&mut self, _mail: Mail) -> Option<Mail> {
        Some(Msg::from_text("Start up signal received").into())
    }

    ///
    ///Pre-shutdown signal that the actor will receive at normal shutdown or  actor
    ///eviction due to actor panicking while processing message. An actor is allowed to
    ///panic a set number of times(currently 3) before eviction. Actor might panic due to
    ///internal(faulty logic, index bounds exception) or external reasons like message
    ///getting corrupted in transit.
    ///
    fn pre_shutdown(&mut self, _mail: Mail) -> Option<Mail> {
        Some(Msg::from_text("Shutdown signal received").into())
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
    ///should be tagged with a non-colliding `typetag` name in the format
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
