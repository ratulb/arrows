use crate::{Msg, Result};
use serde::{Deserialize, Serialize};
use std::any::{self, Any};
use std::fmt::{self, Debug, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

pub trait Actor: Any + Send + Sync {
    fn receive(&mut self, _msg: Msg) -> Option<Msg> {
        Some(Msg::Blank)
    }
    fn name(&self) -> &'static str {
        any::type_name::<Self>()
    }
    fn post_start(&mut self, _msg: Msg) -> Option<Msg> {
        Some(Msg::new_internal("Actor loading", self.name(), "system"))
    }
    fn pre_shutdown(&mut self, _msg: Msg) -> Option<Msg> {
        Some(Msg::new_internal("Actor unloading", self.name(), "system"))
    }
}

impl Debug for dyn Actor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let debug_msg = format!("Actor impl({:?})", self.name());
        write!(f, "{}", debug_msg)
    }
}

#[typetag::serde]
pub trait ActorBuilder {
    //Only the following needs to be implemented
    fn build(&mut self) -> Box<dyn Actor>;

    fn persist(&self, path: PathBuf) -> Result<()>
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
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FakeActorBuilder;
#[typetag::serde(name = "fake_actor_builder")]
impl ActorBuilder for FakeActorBuilder {
    fn build(&mut self) -> Box<dyn Actor> {
        panic!("Should not be called on FakeActorBuilder");
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
        impl Actor for MyActor {
            fn receive(&mut self, _message: Msg) -> Option<Msg> {
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

        //Tag the impl with distinguishable name
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

    #[test]
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
        let mut builder: Box<dyn ActorBuilder> = FakeActorBuilder::default()
            .from_file(PathBuf::from("my_actor_builder"))
            .unwrap();
        let mut actor: Box<dyn Actor> = builder.build();
        let response = actor.receive(Msg::Blank);
        assert!(response.is_some());
    }
}
