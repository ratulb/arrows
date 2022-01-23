use crate::catalog::PanicWatch;
use crate::Error::{self, RegistrationError, RestorationError};
use crate::{Actor, Addr, Config, Mail, Producer, ProducerDeserializer, RichMail};
use std::any::Any;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub(super) struct Actors {
    actor_cache: HashMap<Addr, CachedActor>,
}
unsafe impl Send for Actors {}
unsafe impl Sync for Actors {}
type OutputChannel = Option<Sender<RichMail>>;

impl Actors {
    pub(super) fn new() -> Self {
        //Set panic handler for for the actors. We don't want to eject actors on the very
        //first instance that it panics. Panics may be due to corrupt messages.
        //Hence we maintain a tolerable count limit.
        //Just initialize it once for setting the actor panic hook
        let _panic_watch = PanicWatch::new();
        Self {
            actor_cache: HashMap::new(),
        }
    }

    pub(super) fn get(&self, addr: &Addr) -> Option<&CachedActor> {
        self.actor_cache.get(addr)
    }

    pub(super) fn get_mut(&mut self, addr: &Addr) -> Option<&mut CachedActor> {
        self.actor_cache.get_mut(addr)
    }

    pub(super) fn add(&mut self, addr: Addr, actor: CachedActor) -> Option<CachedActor> {
        self.actor_cache.insert(addr, actor)
    }

    pub(super) fn remove(actors: &mut Self, addr: &Addr) -> Option<CachedActor> {
        actors.actor_cache.remove(addr).and_then(pre_shutdown)
    }

    pub(super) fn play_registration_acts(
        actors: &mut Self,
        addr: Addr,
        actor: CachedActor,
    ) -> Result<Option<CachedActor>, Error> {
        let evicted = Self::add(actors, addr.clone(), actor).and_then(pre_shutdown);
        let admitted = Self::remove(actors, &addr).and_then(post_start);
        match admitted {
            Some(admitted) => {
                Self::add(actors, addr, admitted);
                Ok(evicted)
            }
            None => Err(RegistrationError),
        }
    }
    pub(super) fn play_restoration_acts(
        actors: &mut Self,
        addr: Addr,
        actor: CachedActor,
    ) -> Result<Option<CachedActor>, Error> {
        Self::play_registration_acts(actors, addr, actor).map_err(|_| RestorationError)
    }
}
type Binary = Box<dyn Actor>;

#[derive(Debug)]
pub struct CachedActor {
    binary: Option<Binary>,
    sequence: i64,
    outputs: Vec<Option<Mail>>,
    channel: OutputChannel,
    addr: Addr,
}

impl CachedActor {
    pub(crate) fn new(text: &str, addr: Addr, channel: OutputChannel) -> Option<Self> {
        let producer = ProducerDeserializer::default().from_string(text.to_string());
        match producer {
            Ok(mut producer) => {
                let actor: Binary = producer.produce();
                Some(Self {
                    binary: Some(actor),
                    sequence: 0,
                    outputs: Vec::new(),
                    channel,
                    addr,
                })
            }
            Err(err) => {
                eprintln!("Error creating CachedActor: {}", err);
                None
            }
        }
    }

    pub(crate) fn should_handle_message(actor: &CachedActor, mail: &RichMail) -> bool {
        actor.sequence <= mail.seq()
    }

    pub(crate) fn get_addr(&self) -> &Addr {
        &self.addr
    }

    pub(crate) fn get_sequence(actor: &CachedActor) -> i64 {
        actor.sequence
    }

    pub(crate) fn get_sequence_mut(actor: &mut CachedActor) -> &mut i64 {
        &mut actor.sequence
    }
    pub(crate) fn increment_sequence(actor_seq: &mut i64) {
        *actor_seq += 1;
    }

    pub(crate) fn set_sequence(actor_seq: &mut i64, seq: i64) {
        *actor_seq = seq;
    }

    pub(crate) fn is_loaded(actor: &CachedActor) -> bool {
        actor.binary.is_some()
    }

    /***pub(crate) fn re_define_self(&mut self, text: &str) -> bool {
        let re_incarnate = Self::new(text, Addr::default(), None);
        match re_incarnate {
            Some(mut re_incarnate) => {
                re_incarnate.outputs = std::mem::take(&mut self.outputs);
                re_incarnate.sequence = self.sequence;
                re_incarnate.addr = self.addr.clone();
                re_incarnate.channel = self.channel.take();
                *self = re_incarnate;
                true
            }
            None => false,
        }
    }***/

    pub(crate) fn take_over_from(this: &mut CachedActor, other: &CachedActor) {
        this.sequence = other.sequence;
        this.outputs = other.outputs.clone();
        this.addr = other.addr.clone();
        this.channel = other.channel.clone();
    }

    pub(crate) fn receive(
        actor: &mut CachedActor,
        mut mail: RichMail,
    ) -> Result<(), Box<dyn Any + Send + 'static>> {
        if !CachedActor::is_loaded(actor) || !CachedActor::should_handle_message(actor, &mail) {
            return Ok(());
        }
        if let Some(ref mut binary) = CachedActor::actor_binary(actor) {
            let rs = Self::execute(binary, mail.mail_out());
            match rs {
                Ok(mut outcome) => {
                    Mail::set_from(&mut outcome, CachedActor::get_addr(actor));
                    CachedActor::push_outcome(CachedActor::output_buffer(actor), outcome);
                    CachedActor::increment_sequence(CachedActor::get_sequence_mut(actor));
                    /***println!(
                        "CachedActor current message seq {:?}",
                        CachedActor::get_sequence_mut(actor)
                    );***/
                    Self::flush_buffer(actor);
                }
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    fn flush_buffer(actor: &mut CachedActor) {
        if CachedActor::should_flush(CachedActor::buffer_size(actor)) {
            let buffered = std::mem::take(CachedActor::output_buffer(actor));
            if let Some(ref channel) = actor.channel {
                channel
                    .send(RichMail::RichContent(
                        Mail::fold(buffered),
                        false,
                        CachedActor::get_sequence(actor),
                        Some(CachedActor::get_addr(actor).clone()),
                        None,
                    ))
                    .expect("Published output");
            }
        }
    }

    fn execute(
        binary: &mut Binary,
        mail: Mail,
    ) -> Result<Option<Mail>, Box<dyn Any + Send + 'static>> {
        match catch_unwind(AssertUnwindSafe(|| binary.receive(mail))) {
            Ok(outcome) => Ok(outcome),
            Err(err) => {
                eprintln!("{:?}", err);
                Err(err)
            }
        }
    }

    pub(crate) fn push_outcome(output_buffer: &mut Vec<Option<Mail>>, mail: Option<Mail>) {
        if mail.is_some() {
            output_buffer.push(mail);
        }
    }

    pub(crate) fn actor_binary(actor: &mut CachedActor) -> &mut Option<Binary> {
        &mut actor.binary
    }

    pub(crate) fn should_flush(buffer_size: usize) -> bool {
        buffer_size >= Config::get_shared().db_buff_size()
    }

    pub(crate) fn buffer_size(actor: &CachedActor) -> usize {
        actor.outputs.len()
    }

    pub(crate) fn output_buffer(actor: &mut CachedActor) -> &mut Vec<Option<Mail>> {
        &mut actor.outputs
    }
}

//Pre-shutdown message
fn pre_shutdown(mut actor: CachedActor) -> Option<CachedActor> {
    let _ignored = CachedActor::receive(
        &mut actor,
        RichMail::RichContent(Mail::Blank, true, 0, None, None),
    );
    println!("Pre shutdown hook fired for actor ({})", actor.get_addr().get_name());
    Some(actor)
}
//Post startup message
fn post_start(mut actor: CachedActor) -> Option<CachedActor> {
    let _ignored = CachedActor::receive(
        &mut actor,
        RichMail::RichContent(Mail::Blank, true, 0, None, None),
    );
    println!("Post start hook fired for actor ({})", actor.get_addr().get_name());
    Some(actor)
}
