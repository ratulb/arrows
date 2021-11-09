use crate::common::{actor::Actor, actor::ActorBuilder, msg::Msg};
use crate::registry::registry::ctxops::*;
use crate::registry::storage::StorageContext;
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
    pub(crate) arrows: Arrows,
    pub(crate) storage: StorageContext,
}

impl Context {
    pub fn init() -> Context {
        let arrows = Arrows::new();
        let mut storage = StorageContext::new();
        storage.setup();
        Self { arrows, storage }
    }
}

#[derive(Debug)]
pub(crate) struct Arrows {
    pub(crate) wrappers: HashMap<u64, Rc<RefCell<Box<dyn Actor>>>>,
}
unsafe impl Send for Arrows {}
unsafe impl Sync for Arrows {}

impl Arrows {
    pub(crate) fn new() -> Self {
        Self {
            wrappers: HashMap::new(),
        }
    }
    pub(crate) fn get_actor(&self, identity: u64) -> Option<RefMut<'_, Box<dyn Actor>>> {
        self.wrappers
            .get(&identity)
            .as_mut()
            .map(|entry| entry.borrow_mut())
    }
    pub(crate) fn add_actor(&mut self, identity: u64, rc_actor: Rc<RefCell<Box<dyn Actor>>>) {
        self.wrappers.insert(identity, rc_actor.clone());
    }
    pub(crate) fn remove_actor(&mut self, identity: u64) -> Option<Rc<RefCell<Box<dyn Actor>>>> {
        println!("System startup check 7  - Removing actor");
        self.wrappers.remove(&identity)
    }
}

pub fn register(
    addr: u64,
    mut builder: impl ActorBuilder,
) -> Result<Rc<RefCell<Box<dyn Actor>>>, Error> {
    //Will replace existing
    //Will store name on db
    //Will create inbox and outbox
    //Will fire pre shutdown on existing -
    //Will schedule it
    //Will send post_start message - configure post start message

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

pub(in crate::registry::registry) mod ctxops {
    use super::*;
    pub(super) fn send_msg(identity: u64, msg: Msg) {
        let ctx = CTX.write().unwrap();
        let actor = ctx.arrows.get_actor(identity);
        if let Some(mut actor) = actor {
            actor.receive(msg);
            println!("Msg delivered");
        } else {
            eprintln!("Actor not found");
        }
    }

    pub(super) fn remove_actor(identity: u64) -> Option<Rc<RefCell<Box<dyn Actor>>>> {
        CTX.write().unwrap().arrows.remove_actor(identity)
    }

    //Send a shutdown msg to the actor that is being removed
    pub(super) fn pre_shutdown(actor: Rc<RefCell<Box<dyn Actor>>>) -> Option<()> {
        let _ignored = actor.borrow_mut().receive(Msg::Blank);
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

    pub(super) fn add_actor(
        addr: u64,
        actor: Box<dyn Actor>,
    ) -> Option<Rc<RefCell<Box<dyn Actor>>>> {
        let actor = Rc::new(RefCell::new(actor));
        CTX.write().unwrap().arrows.add_actor(addr, actor.clone());
        Some(actor)
    }

    pub(super) fn post_start(
        actor: Rc<RefCell<Box<dyn Actor>>>,
    ) -> Option<Rc<RefCell<Box<dyn Actor>>>> {
        let _ignored = actor.borrow_mut().receive(Msg::Blank);
        Some(actor)
    }
}
/***#[macro_export]
macro_rules! register {
    ($actor_type:ty, $creator:path) => {{
        //pub extern "C" fn create_actor() -> *mut dyn arrows_common::Actor {
        //pub fn create_actor() -> Box<dyn arrows_common::Actor> {
        let creator: fn() -> $actor_type = $creator;
        let actor = creator();
        println!("I am getting called here");
        let boxed_actor: Box<dyn common::Actor> = Box::new(actor);
        //Box::into_raw(boxed_actor)
        // let write_lock_result = ARROWS.write();
        //let mut arrows = write_lock_result.unwrap();
        //println!("The arrows: {:?}", arrows);
        // println!("The arrows: {:?}", arrows);
        boxed_actor
        //}
        //create_actor()
    }};
}***/

pub(crate) struct Arrow {
    inner: Option<Box<dyn Actor>>,
}

impl Arrow {
    pub(crate) fn new() -> Self {
        Self { inner: None }
    }
    /*** pub(crate) fn inner(&self) -> Option<&dyn Actor> {
        self.inner.as_ref()
    }***/
    pub(crate) fn set(&mut self, actor: Box<dyn Actor>) {
        //let _ignored = replace(&mut self.inner, Some(actor));
        self.inner.as_mut().map(|_| actor);
    }
}
