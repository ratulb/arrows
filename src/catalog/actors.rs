use crate::catalog::ActorRef;
use crate::catalog::ActorRefMut;
use crate::catalog::CachedActor;

use crate::Addr;

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

    pub(super) fn get_actor(&self, addr: &Addr) -> ActorRef<'_> {
        self.actor_cache.get(addr).map(|entry| entry.borrow())
    }

    pub(super) fn get_actor_mut(&self, addr: &Addr) -> ActorRefMut<'_> {
        self.actor_cache.get(addr).map(|entry| entry.borrow_mut())
    }

    pub(super) fn add_actor(&mut self, addr: Addr, actor: CachedActor) -> Option<CachedActor> {
        self.actor_cache.insert(addr, actor.clone());
        Some(actor)
    }

    pub(super) fn remove_actor(&mut self, addr: &Addr) -> Option<CachedActor> {
        self.actor_cache.remove(addr)
    }
}
