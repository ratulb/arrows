//! Catalog
//! Manages a pool of actors, exposes an API [define_actor()] for defining actors in the 
//! system.
//!Provides internal apis for activatation/restoration of actors, a handle to backend store.

mod actors;
mod panics;
use crate::apis::Store;
use crate::catalog::actors::{Actors, CachedActor};
use crate::catalog::panics::PanicWatch;
use crate::common::{actor::Producer, mail::Mail};
use crate::events::DBEvent;
use crate::routing::messenger::Messenger;
use crate::Error::{self, RegistrationError, RestorationError};
use crate::{Addr, RichMail};
use lazy_static::lazy_static;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use std::cell::RefCell;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{channel, Receiver, Sender},
    Arc,
};
use std::thread::JoinHandle;
use std::time::Duration;

lazy_static! {
    pub(crate) static ref CTX: Arc<ReentrantMutex<RefCell<Context>>> =
        Arc::new(ReentrantMutex::new(Context::init()));
}
///Maintains a pool of actors, a handle to the backing store, exposes an API for
///defining(registering) new actor [Producer](crate::Producer).
#[derive(Debug)]
pub struct Context {
    actors: Actors,
    store: Store,
    handle: Option<JoinHandle<()>>,
    dispatcher: Dispatcher,
}
type Dispatcher = Sender<RichMail>;

impl Context {
    pub(crate) fn init() -> RefCell<Self> {
        let actors = Actors::new();
        let mut store = Store::new();
        if let Err(err) = store.setup() {
            panic!("{}", err);
        }
        let ctx_init = Arc::new(AtomicBool::new(true));
        let init_watch = Arc::clone(&ctx_init);
        let (dispatcher, channel): (Sender<RichMail>, Receiver<_>) = channel();
        let handle = std::thread::spawn(move || {
            while init_watch.load(Ordering::Acquire) {}
            //Let there be Context
            std::thread::sleep(Duration::from_millis(10));
            loop {
                match channel.recv() {
                    Ok(mut rich_mail) => {
                        let mail: Mail = rich_mail.mail_out();
                        let inouts = Mail::split(mail);
                        if let Some((ins, outs)) = inouts {
                            if !ins.is_empty() {
                                rich_mail.replace_mail(ins);
                                self::egress(rich_mail);
                            }
                            if !outs.is_empty() {
                                if let Err(err) = Messenger::mail(Mail::Bulk(outs)) {
                                    eprintln!("{:?}", err);
                                }
                            }
                        }
                    }
                    Err(err) => eprintln!("{}", err),
                }
            }
        });
        let ctx = RefCell::new(Self {
            actors,
            store,
            handle: Some(handle),
            dispatcher,
        });
        ctx_init.store(false, Ordering::Release);
        ctx
    }
    ///Ingress incoming mail to the backing store
    pub fn ingress(&mut self, payload: Mail) -> std::io::Result<Option<Mail>> {
        match self.store.persist(payload) {
            Ok(_) => Ok(None),
            Err(err) => {
                eprintln!("{}", err);
                Ok(Some(Mail::Blank))
            }
        }
    }

    pub(crate) fn egress(&mut self, mail: RichMail) {
        let _rs = self.store.egress(mail);
    }

    //Numeric identity of the actor
    pub(crate) fn remove_actor_permanent(&mut self, identity: &str) -> Result<(), Error> {
        self.store
            .remove_actor_permanent(identity)
            .map_err(|err| Error::Other(Box::new(err)))
    }
    //Save an actor producer defintion in the backing store. Current active actor, if any, will
    //not be disturbed
    pub(crate) fn save_producer(
        &mut self,
        identity: &str,
        addr: Addr,
        producer: &impl Producer,
    ) -> Result<(), Error> {
        let text = serde_json::to_string(producer as &dyn Producer)?;
        self.store
            .save_producer(identity, addr, &text)
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
    //Defines an actor in the system. The producer instantiates actors.
    pub(crate) fn define_actor(
        &mut self,
        addr: Addr,
        producer: impl Producer,
    ) -> Result<Option<CachedActor>, Error> {
        let text = serde_json::to_string(&producer as &dyn Producer)?;
        match CachedActor::new(&text, addr.clone(), Some(self.dispatcher.clone())) {
            Some(mut actor) => {
                let previous = Actors::get(&self.actors, &addr);
                let identity = addr.get_id().to_string();
                if let Some(previous) = previous {
                    CachedActor::take_over_from(&mut actor, previous);
                    let _rs = self.remove_actor_permanent(&identity);
                }
                self.save_producer(&identity.to_string(), addr.clone(), &producer)?;
                Actors::play_registration_acts(&mut self.actors, addr, actor)
            }
            None => Err(RegistrationError),
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
                match CachedActor::new(&text, addr.clone(), Some(self.dispatcher.clone())) {
                    Some(mut actor) => {
                        CachedActor::set_sequence(
                            CachedActor::get_sequence_mut(&mut actor),
                            msg_seq,
                        );
                        Actors::play_restoration_acts(&mut self.actors, addr, actor)
                    }
                    None => Err(RestorationError),
                }
            }
            None => Err(RestorationError),
        }
    }

    fn is_actor_defined(&mut self, addr: &Addr) -> bool {
        match Actors::get(&self.actors, addr) {
            Some(_) => true,
            None => {
                let rs = self.restore(addr.clone());
                rs.is_ok() && rs.ok().is_some()
            }
        }
    }

    pub(crate) fn handle_invocation(&mut self, rich_mail: RichMail) {
        let addr = rich_mail.to();
        if let Some(ref addr_inner) = addr {
            let defined = self.is_actor_defined(addr_inner);
            if !defined {
                eprintln!(
                    "Actor definition not found in the system for :{}!",
                    addr_inner
                );
                return;
            }
            let mut actor_addr = Addr::default();
            let actor_id = addr_inner.get_id();
            PanicWatch::set_watch(actor_id);
            let mut panicked = false;
            {
                let actor = self.actors.get_mut(addr_inner);
                if let Some(actor) = actor {
                    if let Err(_err) = CachedActor::receive(actor, rich_mail) {
                        panicked = PanicWatch::has_exceeded_tolerance(actor_id);
                        actor_addr = CachedActor::get_addr(actor).clone();
                    }
                }
            }
            if panicked {
                println!(
                    "Actor panic count {}. Has exceeded tolerance ({:?}). Removing.",
                    PanicWatch::count(actor_id),
                    PanicWatch::tolerance()
                );
                Actors::remove(&mut self.actors, &actor_addr);
                PanicWatch::remove_watch(&actor_id);
            }
        }
    }

    //Exclusive mutable handle to Context - sigleton lock.
    pub(crate) fn handle() -> ReentrantMutexGuard<'static, RefCell<Context>> {
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
///[catalog]:define_actor
///Define an actor in the system providing the actor address(Addr) and an implmentation of
///`Producer`. Existing actor with the same address, if any, would be removed and a
///pre-shutdown signal would be sent to the removed instance. New actor would be added to
///a pool of in memory actors and it would receive a startup signal. Supplied producer
///definition would be peristed in the backing store. On restart - actors will be restored
///on demand to process pending or incoming messages. Actors will restart from where they
///left off.

pub fn define_actor(addr: Addr, producer: impl Producer) -> Result<Option<CachedActor>, Error> {
    Context::handle().borrow_mut().define_actor(addr, producer)
}

pub(crate) fn ingress(mail: Mail) -> std::io::Result<Option<Mail>> {
    Context::handle().borrow_mut().ingress(mail)
}
pub(crate) fn egress(mail: RichMail) {
    Context::handle().borrow_mut().egress(mail);
}
pub(crate) fn restore(addr: Addr) -> Result<Option<CachedActor>, Error> {
    Context::handle().borrow_mut().restore(addr)
}
pub(crate) fn handle_invocation(mail: RichMail) {
    Context::handle().borrow_mut().handle_invocation(mail);
}
