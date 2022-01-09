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
#[derive(Debug, Default)]
pub struct CachedActor {
    exe: Option<Box<dyn Actor>>,
    pub(crate) sequence: i64,
    outputs: Vec<Option<Mail>>,
}

impl CachedActor {
    pub(crate) fn new(text: &str) -> Option<Self> {
        let builder = BuilderDeserializer::default().from_string(text.to_string());
        match builder {
            Ok(mut builder) => {
                let actor: Box<dyn Actor> = builder.build();
                Some(Self {
                    exe: Some(actor),
                    sequence: 0,
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
                re_incarnate.outputs = std::mem::take(&mut self.outputs);
                re_incarnate.sequence = self.sequence;
                *self = re_incarnate;
                true
            }
            None => false,
        }
    }

    pub(crate) fn from(text: &str, mut other: Option<&mut Self>) -> Option<Self> {
        let mut new_actor = Self::new(text);
        if let Some(ref mut materialized) = new_actor {
            if let Some(state) = other.take() {
                materialized.sequence = state.sequence;
                materialized.outputs = std::mem::take(&mut state.outputs);
            }
        }
        if new_actor.is_some() {
            new_actor
        } else {
            other
                .take()
                .map(std::mem::take)
        }
    }

    pub(crate) fn attributes_from(&mut self, other: &CachedActor) {
        self.sequence = other.sequence;
        self.outputs = other.outputs.clone();
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
