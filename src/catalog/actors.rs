use crate::constants::ACTOR_BUFFER_SIZE;
use crate::{Actor, Addr, Mail, Producer, ProducerDeserializer, RichMail};
use std::collections::HashMap;

#[derive(Debug)]
pub(super) struct Actors {
    pub(crate) actor_cache: HashMap<Addr, CachedActor>,
}
unsafe impl Send for Actors {}
unsafe impl Sync for Actors {}

impl Actors {
    pub(super) fn new() -> Self {
        Self {
            actor_cache: HashMap::new(),
        }
    }

    pub(super) fn get_actor(&self, addr: &Addr) -> Option<&CachedActor> {
        self.actor_cache.get(addr)
    }

    pub(super) fn get_actor_mut(&mut self, addr: &Addr) -> Option<&mut CachedActor> {
        self.actor_cache.get_mut(addr)
    }

    pub(super) fn add_actor(&mut self, addr: Addr, actor: CachedActor) -> Option<CachedActor> {
        self.actor_cache.insert(addr, actor)
    }

    pub(super) fn remove_actor(&mut self, addr: &Addr) -> Option<CachedActor> {
        self.actor_cache.remove(addr)
    }
}
#[derive(Debug)]
pub struct CachedActor {
    exe: Option<Box<dyn Actor>>,
    sequence: i64,
    outputs: Vec<Option<Mail>>,
}

impl CachedActor {
    pub(crate) fn new(text: &str) -> Option<Self> {
        let builder = ProducerDeserializer::default().from_string(text.to_string());
        match builder {
            Ok(mut builder) => {
                let actor: Box<dyn Actor> = builder.build();
                Some(Self {
                    exe: Some(actor),
                    sequence: 0,
                    outputs: Vec::new(),
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
        let re_incarnate = Self::new(text);
        match re_incarnate {
            Some(mut re_incarnate) => {
                re_incarnate.outputs = std::mem::take(&mut self.outputs);
                re_incarnate.sequence = self.sequence;
                *self = re_incarnate;
                true
            }
            None => false,
        }
    }

    pub(crate) fn take_over_from(this: &mut CachedActor, other: &CachedActor) {
        this.sequence = other.sequence;
        this.outputs = other.outputs.clone();
    }

    pub(crate) fn receive(actor: &mut CachedActor, mut mail: RichMail) {
        if !CachedActor::is_loaded(actor) || !CachedActor::should_handle_message(actor, &mail) {
            return;
        }
        match CachedActor::actor_exe(actor) {
            Some(ref mut executable) => {
                let outcome = executable.receive(mail.mail_out());
                CachedActor::push_outcome(CachedActor::output_buffer(actor), outcome);
                CachedActor::increment_sequence(CachedActor::get_sequence_mut(actor));
                println!(
                    "CachedActor current message seq {:?}",
                    CachedActor::get_sequence_mut(actor)
                );
                println!(
                    "CachedActor buffer size {:?}",
                    CachedActor::get_sequence(actor)
                );
                if CachedActor::should_flush(CachedActor::buffer_size(actor)) {
                    //Do flush here
                    let _buffered = std::mem::take(CachedActor::output_buffer(actor));
                    //TODO egress
                    //ingress(Mail::Bulk(buffered));
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
