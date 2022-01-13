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
//ProducerDeserializer is used to create actor producers from their serialized state.

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ProducerDeserializer;

#[typetag::serde(name = "builder_deserializer")]
impl Producer for ProducerDeserializer {
    fn build(&mut self) -> Box<dyn Actor> {
        panic!("Should not be called on ProducerDeserializer");
    }
}
//Sample actor and actor producer
pub struct ExampleActor;
impl Actor for ExampleActor {
    fn receive(&mut self, incoming: Mail) -> std::option::Option<Mail> {
        match incoming {
            Mail::Trade(mut msg) => {
                println!("Actor received msg = {:?}", msg);
                msg.uturn_with_text("Actor reply");
                Some(msg.into())
            }
            _ => None,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExampleActorProducer;
#[typetag::serde(name = "example_actor_producer")]
impl Producer for ExampleActorProducer {
    fn build(&mut self) -> Box<dyn Actor> {
        Box::new(ExampleActor)
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
        let rs = define_actor!("example_actor1", producer);
        println!("The registration result is = {:?}", rs);

        let _rs = define_actor!(Addr::new("example_actor2"), ExampleActorProducer);

        let m = Msg::default();
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
