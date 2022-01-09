use crate::catalog::{ActorRef, ActorRefMut, ActorWrapper};
use crate::{Actor, ActorBuilder, Addr, BuilderDeserializer, Mail};
use std::collections::HashMap;

#[derive(Debug)]
pub(super) struct Actors {
    pub(crate) actor_cache: HashMap<Addr, ActorWrapper>,
}
unsafe impl Send for Actors {}
unsafe impl Sync for Actors {}

impl Actors {
    pub(super) fn new() -> Self {
        Self {
            actor_cache: HashMap::new(),
        }
    }

    pub(super) fn get_actor(&self, addr: &Addr) -> ActorRef<'_> {
        self.actor_cache.get(addr).map(|entry| entry.borrow())
    }

    pub(super) fn get_actor_mut(&self, addr: &Addr) -> ActorRefMut<'_> {
        self.actor_cache.get(addr).map(|entry| entry.borrow_mut())
    }

    pub(super) fn add_actor(&mut self, addr: Addr, actor: ActorWrapper) -> Option<ActorWrapper> {
        self.actor_cache.insert(addr, actor.clone());
        Some(actor)
    }

    pub(super) fn remove_actor(&mut self, addr: &Addr) -> Option<ActorWrapper> {
        self.actor_cache.remove(addr)
    }
}

pub(crate) struct CachedActor {
    loaded: bool,
    defined: bool,
    exe: Option<Box<dyn Actor>>,
    definition: Option<String>,
}

impl CachedActor {
    pub(crate) fn new(text: &str) -> Option<Self> {
        let builder = BuilderDeserializer::default().from_string(text.to_string());
        match builder {
            Ok(mut builder) => {
                let actor: Box<dyn Actor> = builder.build();
                Some(Self {
                    loaded: true,
                    defined: true,
                    exe: Some(actor),
                    definition: Some(String::from(text)),
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

    pub(crate) fn is_defined(&self) -> bool {
        self.definition.is_some()
    }

    pub(crate) fn re_define(&mut self, text: &str) -> bool {
        let re_incarnate = Self::new(text);
        match re_incarnate {
            Some(re_incarnate) => {
                *self = re_incarnate;
                true
            }
            None => false,
        }
    }

    pub(crate) fn execute(&mut self, mail: Mail) -> Option<Mail> {
        if !self.is_loaded() || !self.is_defined() {
            return None;
        }
        match self.exe {
            Some(ref mut executable) => executable.receive(mail),
            None => None,
        }
    }
}
