mod actors;
use crate::apis::Store;
use crate::catalog::ctxops::*;
use crate::common::{
    actor::Actor,
    actor::ActorBuilder,
    mail::{Mail, Msg},
};

use crate::{Addr, BuilderDeserializer, Error};
use lazy_static::lazy_static;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use std::cell::{Ref, RefCell, RefMut};

use std::rc::Rc;
use std::sync::Arc;

use crate::catalog::actors::Actors;

lazy_static! {
    pub static ref CTX: Arc<ReentrantMutex<RefCell<Context>>> =
        Arc::new(ReentrantMutex::new(RefCell::new(Context::init())));
}

type CachedActor = Rc<RefCell<Box<dyn Actor>>>;
type ActorRef<'a> = Option<Ref<'a, Box<dyn Actor>>>;
type ActorRefMut<'a> = Option<RefMut<'a, Box<dyn Actor>>>;

#[derive(Debug)]
pub struct Context {
    actors: Actors,
    store: Store,
}

impl Context {
    pub fn init() -> Self {
        let actors = Actors::new();
        let mut store = Store::new();
        store.setup();
        Self { actors, store }
    }

    pub(crate) fn get_actor(&self, addr: &Addr) -> ActorRef<'_> {
        self.actors.get_actor(addr)
    }

    pub(crate) fn get_actor_mut(&self, addr: &Addr) -> ActorRefMut<'_> {
        self.actors.get_actor_mut(addr)
    }
    pub(crate) fn add_actor(&mut self, addr: Addr, actor: CachedActor) -> Option<CachedActor> {
        self.actors.add_actor(addr, actor)
    }

    pub(crate) fn remove_actor(&mut self, addr: &Addr) -> Option<CachedActor> {
        self.actors.remove_actor(addr)
    }
    //cargo run --example - TODO this need to be changed to support remoting - only messages
    //destined to local system should be looped back
    pub fn send_off(&mut self, payload: Mail) {
        self.store.persist(payload);
    }
    //Numeric identity of the actor
    pub(crate) fn remove_actor_permanent(&mut self, identity: &str) -> Result<(), Error> {
        self.store
            .remove_actor_permanent(identity)
            .map_err(|err| Error::Other(Box::new(err)))
    }
    //Save an actor builder defintion in the backing store. Current active actor, if any, will
    //not be disturbed
    pub(crate) fn save_builder(
        &mut self,
        identity: &str,
        addr: Addr,
        builder: &impl ActorBuilder,
    ) -> Result<(), Error> {
        let builder_def = serde_json::to_string(builder as &dyn ActorBuilder)?;
        self.store
            .save_builder(identity, addr, &builder_def)
            .map_err(|err| Error::Other(Box::new(err)))
    }
    //identity - numeric string
    pub(crate) fn retrieve_actor_def(&mut self, identity: &str) -> Option<(Addr, String)> {
        let result = self.store.retrieve_actor_def(identity);
        match result {
            Ok(addr_and_def) => addr_and_def,
            Err(err) => {
                eprintln!("Error fetching build def = {:?}", err);
                None
            }
        }
    }
    //Defines an actor in the system. The builder instantiates actors.
    pub(crate) fn define_actor(
        &mut self,
        identity: u64,
        addr: Addr,
        mut builder: impl ActorBuilder,
    ) -> Result<CachedActor, Error> {
        self.remove_actor(&addr).and_then(pre_shutdown);
        let identity = identity.to_string();
        self.remove_actor_permanent(&identity);
        self.save_builder(&identity, addr.clone(), &builder)?;
        let actor: Box<dyn Actor> = builder.build();
        self.add_actor(addr, Rc::new(RefCell::new(actor)))
            .and_then(post_start)
            .ok_or(Error::RegistrationError)
    }

    //Restore an actor from the backing storage. Active actor will be replaced on successful
    //retrieval. Left undisturbed if not found.
    pub fn restore(&mut self, addr: Addr) -> Result<Option<CachedActor>, Error> {
        let identity = addr.get_id().to_string();
        match self.retrieve_actor_def(&identity) {
            Some(saved_acor_def) => {
                let mut builder: Box<dyn ActorBuilder> =
                    BuilderDeserializer::default().from_string(saved_acor_def.1)?;
                let actor: Box<dyn Actor> = builder.build();
                let actor = Rc::new(RefCell::new(actor));
                self.add_actor(addr, actor.clone())
                    .and_then(post_start)
                    .ok_or(Error::ActorReloadError)
                    .map(Some)
            }
            None => Err(Error::ActorReloadError),
        }
    }

    //Exclusive mutable handle to Context - sigleton lock. Discretionary usage advisable
    pub fn handle() -> ReentrantMutexGuard<'static, RefCell<Context>> {
        CTX.lock()
    }
}

pub fn define_actor(
    identity: u64,
    addr: Addr,
    builder: impl ActorBuilder,
) -> Result<CachedActor, Error> {
    Context::handle()
        .borrow_mut()
        .define_actor(identity, addr, builder)
}
/***
retrieve_build_def -> retrieve_actor_def


activate_actor.rs -> restore_actor -> restore
dectivate_actor.rs -> deactivate
purge_actor.rs -> purge
register_actor.rs -> define_actor.rs - define

send_to -> send
builder_of - define_actor -> define
***/
//Send off a payload of messages which could be directed to different actors in local or
//remote systems. Where messages would be delivered is decided on the host field to of the to
//address(Addr) of each message
pub fn send_off(payload: Mail) {
    Context::handle().borrow_mut().send_off(payload);
    println!("Send mail comes here!");
}

pub fn send(identity: u64, msg: Msg) {
    send_msg(identity, msg);
}

pub fn restore(addr: Addr) -> Result<Option<CachedActor>, Error> {
    Context::handle().borrow_mut().restore(addr)
}

//Pre-shutdown message
fn pre_shutdown(actor: CachedActor) -> Option<()> {
    let _ignored = actor.borrow_mut().receive(Mail::Blank);
    None
}
//Post startup message
fn post_start(actor: CachedActor) -> Option<CachedActor> {
    let _post_start_msg = actor.borrow_mut().receive(Mail::Blank);
    Some(actor)
}

pub(in crate::catalog) mod ctxops {
    use super::*;

    pub(super) fn send_msg(_identity: u64, _msg: Msg) {
        /***let mut mutex = CTX.lock();
        if let Some(actor) =  mutex.get_mut().actors.get_actor(identity) {
            actor.receive(msg.into());
            println!("Msg delivered");
        } else {
            eprintln!("Actor not found");
        }***/
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
                let _ctx = CTX.lock();
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
        let identity = "10000";
        let addr = Addr::new(identity);
        for _ in 0..1000000 {
            let x: u32 = rng.gen();
            actors.remove_actor(&addr);
            let actor: Box<dyn Actor> = Box::new(NewActor);
            let actor = Rc::new(RefCell::new(actor));
            actors.add_actor(Addr::new(identity), actor);
            let _actor = actors.get_actor(&addr);
            if x >= 1000 {
                assert!(999 <= x);
            }
        }
    }
}
