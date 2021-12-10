use crate::{Msg, Result};
use serde::{Deserialize, Serialize};
use std::any::{self, Any};
use std::fmt::{self, Debug, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

pub trait Actor: Any + Send + Sync {
    //The method by which actors receive their messages(_msg - being ignored here). Messages are
    //durable - An actor may fail - but when it comes back - it will start receiving message -along
    //with any piled up message that it may have missed
    fn receive(&mut self, _msg: Msg) -> Option<Msg> {
        Some(Msg::Blank)
    }
    //The type implenmenting the this trait
    fn type_name(&self) -> &'static str {
        any::type_name::<Self>()
    }
    //The very first message(_msg) sent to the actor instance prior to its normal msg processing
    fn post_start(&mut self, _msg: Msg) -> Option<Msg> {
        Some(Msg::new(
            Some("Actor loading".as_bytes().to_vec()),
            self.type_name(),
            "system",
        ))
    }
    //Message(_msg - being ingnored)  sent to the actor instance prior to shutdown
    fn pre_shutdown(&mut self, _msg: Msg) -> Option<Msg> {
        Some(Msg::new(
            Some("Actor unloading".as_bytes().to_vec()),
            self.type_name(),
            "system",
        ))
    }
}

impl Debug for dyn Actor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let debug_msg = format!("Actor impl({:?})", self.type_name());
        write!(f, "{}", debug_msg)
    }
}

#[typetag::serde]
pub trait ActorBuilder {
    //This method must be implemented to register an actor implementation. There is a one-to-one
    //corresponds between an 'Actor implementation' and its builder('ActorBuilder'). ActorBuilders are
    //persisted so that actors can be resurrected after a failure or restart. Actor builders are
    //identified by their #[typetag::serde(name = "an_actor_builder")] name. These names should not
    //collide in a running system. In reality - they are peristed to sqlite db.
    fn build(&mut self) -> Box<dyn Actor>;

    /***fn persist(&self, path: PathBuf) -> Result<()>
    where
        Self: Sized,
    {
        let json = serde_json::to_string(self as &dyn ActorBuilder)?;
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(&path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(json.as_bytes())?;
        Ok(())
    }

    fn from_file(&self, path: PathBuf) -> Result<Box<dyn ActorBuilder>> {
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        let builder: Box<dyn ActorBuilder> = serde_json::from_str(&content)?;
        Ok(builder)
    }***/
}
//BuilderResurrector is used to rebuild actor builders from their serialized state.

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BuilderResurrector;

#[typetag::serde(name = "builder_resurrector")]
impl ActorBuilder for BuilderResurrector {
    fn build(&mut self) -> Box<dyn Actor> {
        panic!("Should not be called on BuilderResurrector");
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
            fn receive(&mut self, _msg: Msg) -> Option<Msg> {
                Some(Msg::Blank)
            }
        }
        let mut my_actor = MyActor::new();
        assert!(my_actor.receive(Msg::Blank).is_some());
    }

    #[test]
    fn actor_builder_test_1() {
        struct MyActor1;

        impl Actor for MyActor1 {}

        #[derive(Clone, Debug, Serialize, Deserialize, Default)]
        struct MyActorBuilder1;

        //Tag the impl with distinguishable name - actor builder's name should not collide in
        //each specific running system
        #[typetag::serde(name = "my_actor_builder1")]
        impl ActorBuilder for MyActorBuilder1 {
            fn build(&mut self) -> Box<dyn Actor> {
                Box::new(MyActor1)
            }
        }

        let mut builder = MyActorBuilder1::default();
        let mut built_actor = builder.build();
        //Send a blank message and get a response back
        let actor_response = built_actor.receive(Msg::Blank);
        assert!(actor_response.is_some());
    }

    /***#[test]
    fn actor_builder_persist_test_1() {
        #[derive(Clone, Debug, Serialize, Deserialize, Default)]
        struct MyActorBuilder2;

        struct MyActor2;
        impl Actor for MyActor2 {}

        #[typetag::serde(name = "my_actor_builder2")]
        impl ActorBuilder for MyActorBuilder2 {
            fn build(&mut self) -> Box<dyn Actor> {
                Box::new(MyActor2)
            }
        }
        //Save the builder
        assert!(MyActorBuilder2::default()
            .persist(PathBuf::from("my_actor_builder"))
            .is_ok(),);
        //Pull the actor builder back from disk and create an actor instance
        let mut builder: Box<dyn ActorBuilder> = BuilderResurrector::default()
            .from_file(PathBuf::from("my_actor_builder"))
            .unwrap();
        let mut actor: Box<dyn Actor> = builder.build();
        let response = actor.receive(Msg::Blank);
        assert!(response.is_some());
    }***/
}
