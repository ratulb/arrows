use crate::constants::ACTOR_BUFFER_SIZE;
use crate::Error::{self, RegistrationError, RestorationError};
use crate::{Actor, Addr, Mail, Producer, ProducerDeserializer, RichMail};
use std::collections::HashMap;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub(super) struct Actors {
    pub(crate) actor_cache: HashMap<Addr, CachedActor>,
}
unsafe impl Send for Actors {}
unsafe impl Sync for Actors {}

type OutputChannel = Option<Sender<RichMail>>;

impl Actors {
    pub(super) fn new() -> Self {
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

    pub(super) fn remove(&mut self, addr: &Addr) -> Option<CachedActor> {
        self.actor_cache.remove(addr)
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
#[derive(Debug)]
pub struct CachedActor {
    exe: Option<Box<dyn Actor>>,
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
                let actor: Box<dyn Actor> = producer.build();
                Some(Self {
                    exe: Some(actor),
                    sequence: 0,
                    outputs: Vec::new(),
                    addr,
                    channel,
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

    pub(crate) fn get_addr(actor: &CachedActor) -> &Addr {
        &actor.addr
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
        actor.exe.is_some()
    }

    pub(crate) fn re_define_self(&mut self, text: &str) -> bool {
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
    }

    pub(crate) fn take_over_from(this: &mut CachedActor, other: &CachedActor) {
        this.sequence = other.sequence;
        this.outputs = other.outputs.clone();
        this.addr = other.addr.clone();
        this.channel = other.channel.clone();
    }

    pub(crate) fn receive(actor: &mut CachedActor, mut mail: RichMail) {
        if !CachedActor::is_loaded(actor) || !CachedActor::should_handle_message(actor, &mail) {
            return;
        }
        match CachedActor::actor_exe(actor) {
            Some(ref mut executable) => {
                let mut outcome = executable.receive(mail.mail_out());
                Mail::set_from(&mut outcome, CachedActor::get_addr(actor));
                CachedActor::push_outcome(CachedActor::output_buffer(actor), outcome);
                CachedActor::increment_sequence(CachedActor::get_sequence_mut(actor));
                println!(
                    "CachedActor current message seq {:?}",
                    CachedActor::get_sequence_mut(actor)
                );
                if CachedActor::should_flush(CachedActor::buffer_size(actor)) {
                    let buffered = std::mem::take(CachedActor::output_buffer(actor));
                    if let Some(ref channel) = actor.channel {
                        channel
                            .send(RichMail::Content(
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
            None => {}
        }
    }

    pub(crate) fn push_outcome(output_buffer: &mut Vec<Option<Mail>>, mail: Option<Mail>) {
        if mail.is_some() {
            output_buffer.push(mail);
        }
    }

    pub(crate) fn actor_exe(actor: &mut CachedActor) -> &mut Option<Box<dyn Actor>> {
        &mut actor.exe
    }

    pub(crate) fn should_flush(buffer_size: usize) -> bool {
        buffer_size >= ACTOR_BUFFER_SIZE
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
        RichMail::Content(Mail::Blank, true, 0, None, None),
    );
    Some(actor)
}
//Post startup message
fn post_start(mut actor: CachedActor) -> Option<CachedActor> {
    let _ignored = CachedActor::receive(
        &mut actor,
        RichMail::Content(Mail::Blank, true, 0, None, None),
    );
    Some(actor)
}
