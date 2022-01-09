use crate::{Actor, ActorBuilder, Addr, BuilderDeserializer, Mail};
use std::collections::HashMap;

#[derive(Debug)]
pub(super) struct Actors {
    pub(crate) actor_cache: HashMap<Addr, CachedActor>,
}
unsafe impl Send for Actors {}
unsafe impl Sync for Actors {}

impl Actors {
    pub(super) fn new() -> Self {
        Self {
            actor_cache: HashMap::new(),
        }
    }

    pub(super) fn get_actor(&self, addr: &Addr) -> Option<&CachedActor> {
        self.actor_cache.get(addr)
    }

    pub(super) fn get_actor_mut(&mut self, addr: &Addr) -> Option<&mut CachedActor> {
        self.actor_cache.get_mut(addr)
    }

    pub(super) fn add_actor(&mut self, addr: Addr, actor: CachedActor) -> Option<CachedActor> {
        self.actor_cache.insert(addr, actor)
    }

    pub(super) fn remove_actor(&mut self, addr: &Addr) -> Option<CachedActor> {
        self.actor_cache.remove(addr)
    }
}
#[derive(Debug)]
pub struct CachedActor {
    exe: Option<Box<dyn Actor>>,
    sequence: i64,
    outputs: Vec<Option<Mail>>,
}

impl CachedActor {
    pub(crate) fn new(text: &str, msg_seq: i64) -> Option<Self> {
        let builder = BuilderDeserializer::default().from_string(text.to_string());
        match builder {
            Ok(mut builder) => {
                let actor: Box<dyn Actor> = builder.build();
                Some(Self {
                    exe: Some(actor),
                    sequence: msg_seq,
                    outputs: Vec::new(),
                })
            }
            Err(err) => {
                eprintln!("Error creating CachedActor: {}", err);
                None
            }
        }
    }

    pub(crate) fn is_loaded(&self) -> bool {
        self.exe.is_some()
    }

    pub(crate) fn re_define(&mut self, text: &str) -> bool {
        let re_incarnate = Self::new(text);
        match re_incarnate {
            Some(mut re_incarnate) => {
                re_incarnate.sequence = self.sequence;
                re_incarnate.outputs = std::mem::take(&mut self.outputs);
                *self = re_incarnate;
                true
            }
            None => false,
        }
    }

    pub(crate) fn receive(&mut self, mail: Mail) -> Option<Mail> {
        if !self.is_loaded() {
            return None;
        }
        match self.exe {
            Some(ref mut executable) => executable.receive(mail),
            None => None,
        }
    }
}
