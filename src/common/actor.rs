use crate::{Mail, Msg};
use serde::{Deserialize, Serialize};
use std::any::{self, Any};
use std::fmt::{self, Debug, Formatter};

pub trait Actor: Any + Send + Sync {
    fn receive(&mut self, _mail: Mail) -> Option<Mail> {
        Some(Mail::Blank)
    }
    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }
    fn post_start(&mut self, _mail: Mail) -> Option<Mail> {
        Some(Msg::new_with_text("Actor loading", self.type_name(), "system").into())
    }
    fn pre_shutdown(&mut self, _mail: Mail) -> Option<Mail> {
        Some(Msg::new_with_text("Actor unloading", self.type_name(), "system").into())
    }
}

impl Debug for dyn Actor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let debug_msg = format!("Actor impl({:?})", self.type_name());
        write!(f, "{}", debug_msg)
    }
}

#[typetag::serde]
pub trait Producer {
    //This method must be implemented to register an actor implementation. There is a one-to-one
    //corresponds between an 'Actor implementation' and its builder('Producer'). Producers are
    //persisted so that actors can be resurrected after a failure or restart. Actor builders are
    //identified by their #[typetag::serde(name = "an_actor_builder")] name. These names should not
    //collide in a running system.
    fn build(&mut self) -> Box<dyn Actor>;
    fn from_string(&self, content: String) -> std::io::Result<Box<dyn Producer>> {
        let builder: Box<dyn Producer> = serde_json::from_str(&content)?;
        Ok(builder)
    }
}
//ProducerDeserializer is used to rebuild actor builders from their serialized state.

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProducerDeserializer;

#[typetag::serde(name = "builder_deserializer")]
impl Producer for ProducerDeserializer {
    fn build(&mut self) -> Box<dyn Actor> {
        panic!("Should not be called on ProducerDeserializer");
    }
}
//Sample actor and actor producer
pub struct NewActor;
impl Actor for NewActor {
    fn receive(&mut self, _incoming: Mail) -> std::option::Option<Mail> {
        Some(Msg::new_with_text("Reply from new actor", "from", "to").into())
    }
}
#[derive(Debug, Serialize, Deserialize, Default)]
struct NewProducer;
#[typetag::serde(name = "new_actor_builder")]
impl Producer for NewProducer {
    fn build(&mut self) -> Box<dyn Actor> {
        Box::new(NewActor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

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
        //A demo actor implementation which responds by blank replies. Its ignoring the incoming
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
    fn actor_builder_test_1() {
        struct MyActor;

        impl Actor for MyActor {}

        #[derive(Clone, Debug, Serialize, Deserialize, Default)]
        struct MyProducer;

        //Tag the impl with distinguishable name - actor builder's name should not collide in
        //each specific running system
        #[typetag::serde(name = "my_actor_builder")]
        impl Producer for MyProducer {
            fn build(&mut self) -> Box<dyn Actor> {
                Box::new(MyActor)
            }
        }

        let mut builder = MyProducer::default();
        let mut built_actor = builder.build();
        println!("The type name is = {:?}", built_actor.type_name());
        //Send a blank message and get a response back
        let actor_response = built_actor.receive(Mail::Blank);
        assert!(actor_response.is_some());
    }
}
