use crate::Message;
use core::fmt::Debug;

pub trait Actor {
    fn receive<'i: 'o, 'o>(&mut self, message: &mut Message<'i>) -> Option<Message<'o>>;
}

impl Debug for dyn Actor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Actor impl")
    }
}

pub struct ActorArrow {}
