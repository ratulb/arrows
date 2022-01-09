mod actors;
use crate::apis::Store;
use crate::common::{actor::Actor, actor::ActorBuilder, mail::Mail};
use crate::events::DBEvent;
use crate::DetailedMsg;
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

type ActorWrapper = Rc<RefCell<Box<dyn Actor>>>;
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
    pub(crate) fn add_actor(&mut self, addr: Addr, actor: ActorWrapper) -> Option<ActorWrapper> {
        self.actors.add_actor(addr, actor)
    }

    pub(crate) fn remove_actor(&mut self, addr: &Addr) -> Option<ActorWrapper> {
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
    ) -> Result<ActorWrapper, Error> {
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
    pub fn restore(&mut self, addr: Addr) -> Result<Option<ActorWrapper>, Error> {
        let identity = addr.get_id().to_string();
        match self.retrieve_actor_def(&identity) {
            Some(saved_acor_def) => {
                let mut builder: Box<dyn ActorBuilder> =
                    BuilderDeserializer::default().from_string(saved_acor_def.1)?;
                let actor: Box<dyn Actor> = builder.build();
                let actor = Rc::new(RefCell::new(actor));
                self.add_actor(addr, actor)
                    .and_then(post_start)
                    .ok_or(Error::ActorReloadError)
                    .map(Some)
            }
            None => Err(Error::ActorReloadError),
        }
    }

    pub(crate) fn is_actor_defined(&mut self, addr: &Addr) -> bool {
        match self.get_actor(addr) {
            Some(_) => true,
            None => {
                //let rs = restore(addr.clone());
                restore(addr.clone());
                //return rs.is_ok() && rs.ok().is_some();
                true
            }
        }
    }

    pub(crate) fn min_msg_seq(&mut self, actor_id: &str) -> Option<(i64, i64, i64)> {
        let result = self.store.min_msg_seq(actor_id);
        match result {
            Ok(inner) => inner,
            Err(err) => {
                eprintln!("Error fetching seq {:?}", err);
                None
            }
        }
    }

    pub(crate) fn update_events(&mut self, row_id: i64) {
        self.store.update_events(row_id);
    }

    pub fn handle_invocation(&mut self, message: DetailedMsg) {
        let msg = message.0;
        let msg_seq = message.2;
        let addr = msg.get_to().as_ref();
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
                                let actor = self.get_actor_mut(addr_inner);
                                match actor {
                                    Some(mut actor) => {
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
        }
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

    pub(crate) fn load_messages(&mut self, rowids: Vec<i64>) -> Vec<DetailedMsg> {
        self.store.from_messages(rowids).expect("Messages")
    }

    pub(crate) fn past_events(&mut self) -> Vec<DetailedMsg> {
        let events = self.store.read_events().expect("Past events");
        self.load_messages(events)
    }
}

pub(crate) fn perist_buffered(events: Vec<DBEvent>) -> Vec<i64> {
    Context::handle().borrow_mut().perist_buffered(events)
}

pub(crate) fn load_messages(rowids: Vec<i64>) -> Vec<DetailedMsg> {
    Context::handle().borrow_mut().load_messages(rowids)
}

pub(crate) fn past_events() -> Vec<DetailedMsg> {
    Context::handle().borrow_mut().past_events()
}

pub fn define_actor(
    identity: u64,
    addr: Addr,
    builder: impl ActorBuilder,
) -> Result<ActorWrapper, Error> {
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

pub fn restore(addr: Addr) -> Result<Option<ActorWrapper>, Error> {
    Context::handle().borrow_mut().restore(addr)
}

pub(crate) fn is_actor_defined(addr: &Addr) -> bool {
    let defined = Context::handle().borrow_mut().is_actor_defined(addr);
    defined
}

pub(crate) fn update_events(row_id: i64) {
    Context::handle().borrow_mut().update_events(row_id);
}

pub(crate) fn min_msg_seq(actor_id: &str) -> Option<(i64, i64, i64)> {
    Context::handle().borrow_mut().min_msg_seq(actor_id)
}

//Pre-shutdown message
fn pre_shutdown(actor: ActorWrapper) -> Option<()> {
    let _ignored = actor.borrow_mut().receive(Mail::Blank);
    None
}
//Post startup message
fn post_start(actor: ActorWrapper) -> Option<ActorWrapper> {
    let _post_start_msg = actor.borrow_mut().receive(Mail::Blank);
    Some(actor)
}

pub fn handle_invocation(message: DetailedMsg) {
    Context::handle().borrow_mut().handle_invocation(message);
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
            let _actor = actors.get_actor_mut(&addr);
            if x >= 1000 {
                assert!(999 <= x);
            }
        }
    }
}
