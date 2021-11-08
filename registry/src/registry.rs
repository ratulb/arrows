use crate::registry::ctxops::*;
use crate::storage::StorageContext;
use chrono::offset::Utc;
use chrono::DateTime;
use common::{Actor, ActorBuilder, Error, Msg};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

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
    pub(crate) fn get(&self, identity: u64) -> Option<RefMut<'_, Box<dyn Actor>>> {
        self.wrappers
            .get(&identity)
            .as_mut()
            .map(|entry| entry.borrow_mut())
    }
    pub(crate) fn add(&mut self, identity: u64, rc_actor: Rc<RefCell<Box<dyn Actor>>>) {
        self.wrappers.insert(identity, rc_actor.clone());
    }
    pub(crate) fn remove_actor(&mut self, identity: u64) -> Option<Rc<RefCell<Box<dyn Actor>>>> {
        println!("System startup check 7  - Removing actor");
        self.wrappers.remove(&identity)
    }
}

//#[no_mangle]

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
    remove_actor(addr).and_then(shutdown);
    remove_actor_permanent(&identity);

    let new_actor: Box<dyn Actor> = builder.build();
    let builder_def = serde_json::to_string(&builder as &dyn ActorBuilder)?;
    CTX.write()
        .unwrap()
        .storage
        .persist_actor(&identity, &builder_def);
    let new_actor = Rc::new(RefCell::new(new_actor));
    CTX.write().unwrap().arrows.add(addr, new_actor.clone());
    //Startup msg
    let _reply = new_actor.borrow_mut().receive(Msg::Blank);
    Ok(new_actor)
}
pub(super) mod ctxops {
    use super::*;
    pub(super) fn remove_actor(identity: u64) -> Option<Rc<RefCell<Box<dyn Actor>>>> {
        CTX.write().unwrap().arrows.remove_actor(identity)
    }
    //Send a shutdown msg to the actor that is being removed
    pub(super) fn shutdown(actor: Rc<RefCell<Box<dyn Actor>>>) -> Option<()> {
        let _ignored = actor.borrow_mut().receive(Msg::Blank);
        None
    }
    pub(super) fn remove_actor_permanent(identity: &String) -> Result<(), rusqlite::Error> {
        CTX.write()
            .unwrap()
            .storage
            .remove_actor_permanent(identity)
    }
}
/***#[macro_export]
macro_rules! register {
    ($actor_type:ty, $creator:path) => {{
        #[no_mangle]
        //pub extern "C" fn create_actor() -> *mut dyn arrows_common::Actor {
        pub fn create_actor() -> Box<dyn arrows_common::Actor> {
            let creator: fn() -> $actor_type = $creator;
            let actor = creator();
            println!("I am getting called here");
            let boxed_actor: Box<dyn arrows_common::Actor> = Box::new(actor);
            //Box::into_raw(boxed_actor)
            let write_lock_result = ARROWS.write();
            let mut arrows = write_lock_result.unwrap();
            println!("The arrows: {:?}", arrows);
            arrows.push(1);
            println!("The arrows: {:?}", arrows);
            boxed_actor
        }
        create_actor()
    }};
}***/
#[macro_export]
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
}

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

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct ActorImpl;

impl Actor for ActorImpl {
    fn receive(&mut self, _: Msg) -> std::option::Option<Msg> {
        let received_time = SystemTime::now();
        let respond_time = received_time.checked_add(Duration::from_millis(2000));
        let datetime: DateTime<Utc> = received_time.into();
        let respond_time: DateTime<Utc> = respond_time.unwrap().into();
        let datetime = datetime.format("%d/%m/%Y %T");
        println!("Now the time is :{}", datetime);
        let mut reply = String::from(&datetime.to_string());
        reply += " and reply time: ";
        reply += &respond_time.to_string();
        Some(Msg::new_with_text(&reply, "from", "to"))
    }
}
struct ActorCreator;
impl ActorCreator {
    fn create_actor() -> ActorImpl {
        ActorImpl
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn register_test_1() {
        let mut actor: Box<dyn Actor> = register!(ActorImpl, ActorCreator::create_actor);
        let response = actor.receive(Msg::Blank);
        println!("Response: {:?}", response);
    }
}
