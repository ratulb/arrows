pub(crate) mod storage;
use crate::common::{
    actor::Actor,
    actor::ActorBuilder,
    mail::{Mail, Msg},
};
use crate::registry::ctxops::*;
use crate::registry::storage::Storage;
use crate::BuilderDeserializer;
use crate::Error;
use lazy_static::lazy_static;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::RwLock;

lazy_static! {
    pub(crate) static ref CTX: RwLock<Context> = RwLock::new(Context::init());
}

#[derive(Debug)]
pub(crate) struct Context {
    pub(crate) actors: Actors,
    pub(crate) storage: Storage,
}

impl Context {
    pub fn init() -> Context {
        let actors = Actors::new();
        let mut storage = Storage::new();
        storage.setup();
        Self { actors, storage }
    }
}

#[derive(Debug)]
pub(crate) struct Actors {
    pub(crate) cached_actors: HashMap<u64, Rc<RefCell<Box<dyn Actor>>>>,
}
unsafe impl Send for Actors {}
unsafe impl Sync for Actors {}

impl Actors {
    pub(crate) fn new() -> Self {
        Self {
            cached_actors: HashMap::new(),
        }
    }
    pub(crate) fn get_actor(&self, identity: u64) -> Option<RefMut<'_, Box<dyn Actor>>> {
        self.cached_actors
            .get(&identity)
            .as_mut()
            .map(|entry| entry.borrow_mut())
    }
    pub(crate) fn add_actor(&mut self, identity: u64, rc_actor: Rc<RefCell<Box<dyn Actor>>>) {
        self.cached_actors.insert(identity, rc_actor.clone());
    }
    pub(crate) fn remove_actor(&mut self, identity: u64) -> Option<Rc<RefCell<Box<dyn Actor>>>> {
        self.cached_actors.remove(&identity)
    }
}

pub fn register_builder(
    addr: u64,
    mut builder: impl ActorBuilder,
) -> Result<Rc<RefCell<Box<dyn Actor>>>, Error> {
    let identity = addr.to_string();
    remove_actor(addr).and_then(pre_shutdown);
    remove_actor_permanent(&identity);

    persist_builder(&identity, &builder)?;

    let actor: Box<dyn Actor> = builder.build();
    add_actor(addr, actor)
        .and_then(post_start)
        .ok_or(Error::RegistrationError)
}

pub fn send(identity: u64, msg: Msg) {
    send_msg(identity, msg);
}

pub fn reload_actor(addr: u64) -> Result<Rc<RefCell<Box<dyn Actor>>>, Error> {
    match retrieve_build_def(&addr.to_string()) {
        Some(s) => {
            let mut builder: Box<dyn ActorBuilder> =
                BuilderDeserializer::default().from_string(s)?;
            let actor: Box<dyn Actor> = builder.build();
            add_actor(addr, actor)
                .and_then(post_start)
                .ok_or(Error::ActorReloadError)
        }
        None => Err(Error::ActorReloadError),
    }
}

pub(in crate::registry) mod ctxops {
    use super::*;
    pub(super) fn send_msg(identity: u64, msg: Msg) {
        let ctx = CTX.write().unwrap();
        let actor = ctx.actors.get_actor(identity);
        if let Some(mut actor) = actor {
            actor.receive(msg.into());
            println!("Msg delivered");
        } else {
            eprintln!("Actor not found");
        }
    }

    pub(super) fn remove_actor(identity: u64) -> Option<Rc<RefCell<Box<dyn Actor>>>> {
        CTX.write().unwrap().actors.remove_actor(identity)
    }

    //Send a shutdown msg to the actor that is being removed
    pub(super) fn pre_shutdown(actor: Rc<RefCell<Box<dyn Actor>>>) -> Option<()> {
        let _ignored = actor.borrow_mut().receive(Mail::Blank);
        None
    }

    pub(super) fn remove_actor_permanent(identity: &String) -> Result<(), Error> {
        CTX.write()
            .unwrap()
            .storage
            .remove_actor_permanent(identity)
            .map_err(|err| Error::Other(Box::new(err)))
    }

    pub(super) fn persist_builder(
        identity: &String,
        builder: &impl ActorBuilder,
    ) -> Result<(), Error> {
        let builder_def = serde_json::to_string(builder as &dyn ActorBuilder)?;
        CTX.write()
            .unwrap()
            .storage
            .persist_builder(identity, &builder_def)
            .map_err(|err| Error::Other(Box::new(err)))
    }
    pub(super) fn retrieve_build_def(identity: &String) -> Option<String> {
        let rs = CTX.write().unwrap().storage.retrieve_build_def(identity);
        match rs {
            Ok(build_def) => build_def,
            Err(err) => {
                eprintln!("Error fetching build def = {:?}", err);
                None
            }
        }
    }

    pub(super) fn add_actor(
        addr: u64,
        actor: Box<dyn Actor>,
    ) -> Option<Rc<RefCell<Box<dyn Actor>>>> {
        let actor = Rc::new(RefCell::new(actor));
        CTX.write().unwrap().actors.add_actor(addr, actor.clone());
        Some(actor)
    }

    pub(super) fn post_start(
        actor: Rc<RefCell<Box<dyn Actor>>>,
    ) -> Option<Rc<RefCell<Box<dyn Actor>>>> {
        let _post_start_msg = actor.borrow_mut().receive(Mail::Blank);
        Some(actor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Actor, Mail, Msg};
    pub struct NewActor;
    impl Actor for NewActor {
        fn receive(&mut self, _incoming: Mail) -> std::option::Option<Mail> {
            Some(Msg::new_with_text("Reply from new actor", "from", "to").into())
        }
    }
    /***
        real    0m0.576s
        user    0m0.562s
        sys     0m0.014s
    ***/
    #[test]
    fn context_add_get_remove_speed_test1() {
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();

        for _ in 0..1000000 {
            let x: u32 = rng.gen();
            if x > 1000 {
                let _ctx = CTX.write().unwrap();
                assert!(999 <= x);
            }
        }
    }
    /***
        n2-standard-4
        CPU platform
        Intel Cascade Lake
        4 vCPUs, 16 GB memory

        ubuntu-pro-1804-bionic

        real    0m2.577s
        user    0m2.555s
        sys     0m0.022s

    ***/
    #[test]
    fn actors_add_get_remove_speed_test1() {
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        let mut actors = Actors::new();
        for _ in 0..1000000 {
            let x: u32 = rng.gen();
            let identity = 10000;
            actors.remove_actor(identity);
            let actor: Box<dyn Actor> = Box::new(NewActor);
            let actor = Rc::new(RefCell::new(actor));
            actors.add_actor(identity, actor.clone());
            let _actor = actors.get_actor(identity);
            if x >= 1000 {
                assert!(999 <= x);
            }
        }
    }
}
