//! #Demos
//!Do all the definition and initialization for capability demonstration. This is due to
//!the fact that the final binary need to be cognizant of defined entitities like actor
//!implementations and their corresponding producers.

use crate::{Actor, Mail, Msg, Producer};
use serde::{Deserialize, Serialize};

///A sample actor
pub struct NewActor;

impl Actor for NewActor {
    fn receive(&mut self, incoming: Mail) -> Option<Mail> {
        match incoming {
            Mail::Trade(msg) => println!("NewActor received: {:?}", msg.as_text()),
            Mail::Bulk(msgs) => println!("NewActor received: {:?}", msgs[0].as_text()),
            Mail::Blank => println!("NewActor received blank"),
        }
        Some(Msg::from_text("Message from NewActor").into())
    }
}

///Produces NewActor instances
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NewProducer;

#[typetag::serde]
impl Producer for NewProducer {
    fn produce(&mut self) -> Box<dyn Actor> {
        Box::new(NewActor)
    }
}

///Another sample actor that sends message back to the sender
pub struct AnotherActor;

impl Actor for AnotherActor {
    fn receive(&mut self, incoming: Mail) -> Option<Mail> {
        if let Mail::Trade(mut msg) = incoming {
            println!("Received msg {:?}", msg.as_text());
            msg.text_reply("Reply from Another actor");
            Some(msg.into())
        } else {
            None
        }
    }
}

///Produces instances of AnotherActor
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AnotherProducer;

#[typetag::serde]
impl Producer for AnotherProducer {
    fn produce(&mut self) -> Box<dyn Actor> {
        Box::new(AnotherActor)
    }
}
