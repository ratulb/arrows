mod actors;
use crate::apis::Store;
use crate::common::{actor::Producer, mail::Mail};
use crate::events::DBEvent;
use crate::RichMail;
use crate::{Addr, Error};
use lazy_static::lazy_static;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use std::cell::RefCell;

use std::sync::Arc;

use crate::catalog::actors::{Actors, CachedActor};

lazy_static! {
    pub(crate) static ref CTX: Arc<ReentrantMutex<RefCell<Context>>> =
        Arc::new(ReentrantMutex::new(RefCell::new(Context::init())));
}

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
        builder: &impl Producer,
    ) -> Result<(), Error> {
        let text = serde_json::to_string(builder as &dyn Producer)?;
        self.store
            .save_builder(identity, addr, &text)
            .map_err(|err| Error::Other(Box::new(err)))
    }
    //identity - numeric string of actor address(Addr)
    pub(crate) fn retrieve_actor_def(&mut self, identity: &str) -> Option<(Addr, String, i64)> {
        let result = self.store.retrieve_actor_def(identity);
        match result {
            Ok(addr_text_seq) => addr_text_seq,
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
        builder: impl Producer,
    ) -> Result<CachedActor, Error> {
        let text = serde_json::to_string(&builder as &dyn Producer)?;
        match CachedActor::new(&text) {
            Some(mut actor) => {
                let previous = self.actors.remove_actor(&addr).and_then(pre_shutdown);
                if let Some(previous) = previous {
                    actor.attributes_from(&previous);
                    let identity = identity.to_string();
                    self.remove_actor_permanent(&identity);
                }
                self.save_builder(&identity.to_string(), addr.clone(), &builder)?;
                self.actors
                    .add_actor(addr, actor)
                    .and_then(post_start)
                    .ok_or(Error::RegistrationError)
            }
            None => Err(Error::RegistrationError),
        }
    }

    //Restore an actor from the backing storage. Active actor will be replaced on successful
    //retrieval. Left undisturbed if not found.
    pub(crate) fn restore(&mut self, addr: Addr) -> Result<Option<CachedActor>, Error> {
        let identity = addr.get_id().to_string();
        match self.retrieve_actor_def(&identity) {
            Some(definition) => {
                let text = definition.1;
                let msg_seq = definition.2;
                match CachedActor::new(&text) {
                    Some(mut actor) => {
                        CachedActor::set_sequence(
                            CachedActor::get_sequence_mut(&mut actor),
                            msg_seq,
                        );
                        self.actors
                            .add_actor(addr, actor)
                            .and_then(post_start)
                            .ok_or(Error::RestorationError)
                            .map(Some)
                    }
                    None => Err(Error::RestorationError),
                }
            }
            None => Err(Error::RestorationError),
        }
    }

    pub(crate) fn is_actor_defined(&mut self, addr: &Addr) -> bool {
        match self.actors.get_actor(addr) {
            Some(_) => true,
            None => {
                let rs = restore(addr.clone());
                rs.is_ok() && rs.ok().is_some()
            }
        }
    }

    pub(crate) fn handle_invocation(&mut self, rich_mail: RichMail) {
        let addr = rich_mail.to();
        if let Some(addr_inner) = addr {
            let _actor_id = addr_inner.get_id().to_string();
        }

        /***let addr = msg.get_to().as_ref();
        match addr {
            Some(addr_inner) => {
                if !is_actor_defined(addr_inner) {
                    eprintln!("Actor not defined ={:?}", addr);
                } else {
                    let actor_id = addr_inner.get_id().to_string();
                    let curr_msg_seq = min_msg_seq(&actor_id);
                    match curr_msg_seq {
                        Some(sequence) => {
                            if sequence.0 < msg_seq {
                                eprintln!("Out of sequence message!");
                            } else {
                                let actor = self.actors.get_actor_mut(addr_inner);
                                match actor {
                                    Some(actor) => {
                                        let invocation_outcome = actor.receive(Mail::Trade(msg));
                                        println!("Invocation outcome = {:?}", invocation_outcome);
                                        update_events(sequence.1);
                                    }
                                    None => {}
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }***/
    }

    //Exclusive mutable handle to Context - sigleton lock. Discretionary usage advisable
    pub fn handle() -> ReentrantMutexGuard<'static, RefCell<Context>> {
        CTX.lock()
    }

    pub(crate) fn perist_buffered(&mut self, events: Vec<DBEvent>) -> Vec<i64> {
        self.store
            .persist_events(events.into_iter())
            .expect("Events persisted")
    }

    pub(crate) fn load_messages(&mut self, rowids: Vec<i64>) -> Vec<RichMail> {
        self.store.from_messages(rowids).expect("Messages")
    }

    pub(crate) fn past_events(&mut self) -> Vec<RichMail> {
        let events = self.store.read_events().expect("Past events");
        self.load_messages(events)
    }
}

pub(crate) fn perist_buffered(events: Vec<DBEvent>) -> Vec<i64> {
    Context::handle().borrow_mut().perist_buffered(events)
}

pub(crate) fn load_messages(rowids: Vec<i64>) -> Vec<RichMail> {
    Context::handle().borrow_mut().load_messages(rowids)
}

pub(crate) fn past_events() -> Vec<RichMail> {
    Context::handle().borrow_mut().past_events()
}

pub fn define_actor(
    identity: u64,
    addr: Addr,
    builder: impl Producer,
) -> Result<CachedActor, Error> {
    Context::handle()
        .borrow_mut()
        .define_actor(identity, addr, builder)
}

//Send off a payload of messages which could be directed to different actors in local or
//remote systems. Where messages would be delivered is decided on the host field to of the to
//address(Addr) of each message
pub fn send_off(payload: Mail) {
    Context::handle().borrow_mut().send_off(payload);
}

pub fn restore(addr: Addr) -> Result<Option<CachedActor>, Error> {
    Context::handle().borrow_mut().restore(addr)
}
//TODO Make Receive(in routing take mail) -> Send mail
pub(crate) fn is_actor_defined(addr: &Addr) -> bool {
    Context::handle().borrow_mut().is_actor_defined(addr)
}

//Pre-shutdown message
fn pre_shutdown(mut actor: CachedActor) -> Option<CachedActor> {
    let _ignored = actor.receive(Mail::Blank);
    Some(actor)
}
//Post startup message
fn post_start(mut actor: CachedActor) -> Option<CachedActor> {
    let _post_start_msg = actor.receive(Mail::Blank);
    Some(actor)
}

pub(crate) fn handle_invocation(message: RichMail) {
    Context::handle().borrow_mut().handle_invocation(message);
}
