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
    //Example
    pub fn send_mail(&mut self, mail: Mail) {
        self.store.persist(mail);
    }
    //Numeric identity of the actor
    pub(crate) fn remove_actor_permanent(&mut self, identity: &str) -> Result<(), Error> {
        self.store
            .remove_actor_permanent(identity)
            .map_err(|err| Error::Other(Box::new(err)))
    }

    pub(crate) fn persist_builder(
        &mut self,
        identity: &str,
        addr: Addr,
        builder: &impl ActorBuilder,
    ) -> Result<(), Error> {
        let builder_def = serde_json::to_string(builder as &dyn ActorBuilder)?;
        self.store
            .persist_builder(identity, addr, &builder_def)
            .map_err(|err| Error::Other(Box::new(err)))
    }

    pub(crate) fn retrieve_build_def(&mut self, identity: &str) -> Option<(Addr, String)> {
        let result = self.store.retrieve_build_def(identity);
        match result {
            Ok(addr_and_def) => addr_and_def,
            Err(err) => {
                eprintln!("Error fetching build def = {:?}", err);
                None
            }
        }
    }

    pub(crate) fn builder_of(
        &mut self,
        identity: u64,
        addr: Addr,
        mut builder: impl ActorBuilder,
    ) -> Result<CachedActor, Error> {
        self.remove_actor(&addr).and_then(Self::pre_shutdown);
        let identity = identity.to_string();
        self.remove_actor_permanent(&identity);
        self.persist_builder(&identity, addr.clone(), &builder)?;
        let actor: Box<dyn Actor> = builder.build();
        self.add_actor(addr, Rc::new(RefCell::new(actor)))
            .and_then(Self::post_start)
            .ok_or(Error::RegistrationError)
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
    pub fn reference() -> ReentrantMutexGuard<'static, RefCell<Context>> {
        CTX.lock()
    }
}

pub fn builder_of(
    identity: u64,
    addr: Addr,
    builder: impl ActorBuilder,
) -> Result<CachedActor, Error> {
    Context::reference().borrow_mut().builder_of(identity, addr, builder)
}

pub fn send_mail(mail: Mail) {
    Context::reference().borrow_mut().send_mail(mail);
    println!("Send mail comes here!");
}

pub fn send(identity: u64, msg: Msg) {
    send_msg(identity, msg);
}

pub fn reload_actor(addr: u64) -> Result<Box<dyn Actor>, Error> {
    match retrieve_build_def(&addr.to_string()) {
        Some(s) => {
            println!("check1");
            let mut builder: Box<dyn ActorBuilder> =
                BuilderDeserializer::default().from_string(s.1)?;
            println!("check2");
            let actor: Box<dyn Actor> = builder.build();
            println!("check3");
            add_actor(addr, actor)
                .and_then(post_start)
                .ok_or(Error::ActorReloadError)
        }
        None => Err(Error::ActorReloadError),
    }
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
