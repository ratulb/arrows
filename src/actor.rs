use crate::Message;
use core::fmt::Debug;
use std::time::Duration;

pub trait Actor {
    fn receive(&mut self, message: &mut Message) -> Option<Message>;
    //Count of unread messages held in memory for the actor
    fn max_in_memory_msg_count(&self) -> u64 {
        1000
    }
    fn msg_max_age(&self) -> Duration {
        Duration::from_secs(3)
    }
}

impl Debug for dyn Actor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Actor impl")
    }
}

pub struct ActorArrow {}
