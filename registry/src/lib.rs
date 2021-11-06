use chrono::offset::Utc;
use chrono::DateTime;
use common::{actor::ActorBuilder, actor::FakeActorBuilder, file_exists, Actor, Error, Msg};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

lazy_static! {
    pub(crate) static ref CTX: RwLock<Context> = RwLock::new(Context::init());
}

#[derive(Debug)]
pub(crate) struct Context {
    pub(crate) arrows: Arrows,
}

impl Context {
    pub fn init() -> Context {
        let arrows = Arrows::new();
        Self { arrows }
    }
}

#[derive(Debug)]
pub(crate) struct Arrows {
    pub(crate) wrappers: HashMap<u64, Rc<RefCell<dyn Actor>>>,
}
unsafe impl Send for Arrows {}
unsafe impl Sync for Arrows {}
impl Arrows {
    pub(crate) fn new() -> Self {
        Self {
            wrappers: HashMap::new(),
        }
    }
    pub(crate) fn get(&self, addr_id: u64) -> Option<RefMut<'_, dyn Actor>> {
        match self.wrappers.get(&addr_id) {
            Some(ref mut entry) => Some(entry.borrow_mut()),
            None => None,
        }
    }
    pub(crate) fn add(&mut self, addr_id: u64, rc_actor: Rc<RefCell<dyn Actor>>) {
        self.wrappers.insert(addr_id, rc_actor.clone());
    }
    pub(crate) fn remove(&mut self, addr_id: u64) -> Option<Rc<RefCell<(dyn Actor)>>> {
        println!("System startup check 7  - Removing actor");
        self.wrappers.remove(&addr_id)
    }
}

//#[no_mangle]
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;

pub fn register(addr_id: u64, mut builder: impl ActorBuilder) -> Result<Box<dyn Actor>, Error> {
    println!("I am getting called alright {} ", addr_id);
    //Will replace existing
    //Will store string representaion on disk
    //Will store name on db
    //Will create inbox and outbox
    //Will fire pre shutdown on existing -
    //Will schedule it
    //Will send post_start message

    /***let file = File::create(addr_id.to_string()).expect("Unable to create file");
    let json = serde_json::to_string(&builder as &dyn ActorBuilder).unwrap();
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &json);

    let file = File::open(addr_id.to_string())?;
    let mut reader = BufReader::new(file);
    let mut builder: Box<dyn ActorBuilder> = serde_json::from_reader(&mut reader)?;
    let actor = builder.build();***/
    let identity: &str = &addr_id.to_string();
    let mut actor;
    if file_exists(identity) {
        let mut default_builder: Box<dyn ActorBuilder> =
            FakeActorBuilder::default().from_file(PathBuf::from(identity))?;
        actor = default_builder.build();
        let resp = actor.receive(Msg::Blank);
        println!("From existing actor: {:?}", resp);
    } else {
        actor = builder.build();
        builder.persist(PathBuf::from(identity))?;
        let resp = actor.receive(Msg::Blank);
        println!("From new actor: {:?}", resp);
    }
    Ok(actor)
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
    pub(crate) fn inner(&self) -> Option<&Box<dyn Actor>> {
        self.inner.as_ref()
    }
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
