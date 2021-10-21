use crate::{Actor, Address, Message, STORE};
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::io::Result;
use std::rc::Rc;

pub(crate) const REQUEST_VALIDATOR: &str = "request-validator";
pub(crate) const ACTOR_INITIALIZER: &str = "actor-initializer";

#[derive(Debug)]
pub(crate) struct SysActors {
    pub(crate) actors: HashMap<u64, Rc<RefCell<dyn Actor>>>,
}
unsafe impl Send for SysActors {}
unsafe impl Sync for SysActors {}

impl SysActors {
    pub(crate) fn new() -> Self {
        Self {
            actors: HashMap::new(),
        }
    }
    pub(crate) fn get(&self, addr_id: u64) -> Option<RefMut<dyn Actor>> {
        match self.actors.get(&addr_id) {
            Some(ref mut yes) => Some(yes.borrow_mut()),
            None => None,
        }
    }
    pub(crate) fn add(&mut self, addr_id: u64, rc_actor: Rc<RefCell<dyn Actor>>) {
        self.actors.insert(addr_id, rc_actor.clone());
    }
    pub(crate) fn remove(&mut self, addr_id: u64) -> Option<Rc<RefCell<(dyn Actor)>>> {
        println!("System startup check 7  - Removing actor");
        self.actors.remove(&addr_id)
    }
}

pub(crate) fn start() {
    let validator = RequestValidator::new();
    println!("System startup check 2 : {} ", validator.identity());
    let initializer = ActorInitializer::new();
    let write_lock_result = STORE.write();
    let mut store = write_lock_result.unwrap();
    store
        .sys_actors
        .add(validator.identity(), Rc::new(RefCell::new(validator)));
    store
        .sys_actors
        .add(initializer.identity(), Rc::new(RefCell::new(initializer)));
}

pub(crate) struct ActorInitializer<'a> {
    addr: Address<'a>,
}
impl<'a> ActorInitializer<'a> {
    pub(crate) fn new() -> Self {
        println!(
            "Actor initializer starting with assumed name of \"{}\"",
            ACTOR_INITIALIZER
        );
        Self {
            addr: Address::new(ACTOR_INITIALIZER),
        }
    }
    pub(crate) fn identity(&self) -> u64 {
        self.addr.get_id()
    }
}

pub(crate) struct ActorInvoker;

impl ActorInvoker {
    pub(crate) fn invoke(mut incoming: Message) -> Result<()> {
        let to_addr_id = incoming.get_to_id();
        println!("System startup check 4 - in invoke {:?}", to_addr_id);
        {
            let read_lock_result = STORE.read();
            let store = read_lock_result.unwrap();
            let mut actor = store.sys_actors.get(to_addr_id);
            if let Some(ref mut actor_ref) = actor {
                let outcome = actor_ref.receive(&mut incoming);
                println!(
                    "System startup check 5(1) - got hold of actor - outcome {:?}",
                    outcome.unwrap().content_as_text()
                );
            } else {
                println!("System startup check 5(2) - could not get actor");
                MsgRouter::route(incoming);
            }
        }
        /***println!("System startup check 6 - Going to remove actor");
        let write_lock_result = STORE.write();
        let mut store = write_lock_result.unwrap();
        store.sys_actors.remove(to_addr_id);***/
        Ok(())
    }
}
pub(crate) struct MsgRouter {}
impl MsgRouter {
    pub(crate) fn route(msg: Message) -> Result<()> {
        Ok(())
    }
}

struct LocalMsgRouter;
struct RemoteMsgRouter;

pub(crate) struct RequestValidator<'a> {
    addr: Address<'a>,
}

impl<'a> RequestValidator<'a> {
    pub(crate) fn new() -> Self {
        println!(
            "Request validator starting with assumed name of \"{}\"",
            REQUEST_VALIDATOR
        );
        Self {
            addr: Address::new(REQUEST_VALIDATOR),
        }
    }

    pub(crate) fn identity(&self) -> u64 {
        self.addr.get_id()
    }
}

impl<'a> Actor for RequestValidator<'a> {
    fn receive<'i: 'o, 'o>(&mut self, incoming: &mut Message<'i>) -> Option<Message<'o>> {
        println!("Received validation message - allowing to proceed");
        incoming.uturn_with_text("Request validation passed");
        let outgoing = std::mem::replace(incoming, Message::Blank);
        Some(outgoing)
    }
}
impl<'a> Actor for ActorInitializer<'a> {
    fn receive<'i: 'o, 'o>(&mut self, _incoming: &mut Message<'i>) -> Option<Message<'o>> {
        None
    }
}
