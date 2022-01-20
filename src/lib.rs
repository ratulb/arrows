#![deny(rust_2018_idioms)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
pub use catalog::{ingress, restore};
pub(crate) use common::actor::ProducerDeserializer;
pub use common::actor::{Actor, ExampleActorProducer, Producer};
pub use common::addr::Addr;
pub use common::errs::{Error, Result};
pub use common::mail::{Action, Mail, Msg};
pub use common::utils::*;
pub use routing::listener::MessageListener;
pub(crate) use store::*;

pub mod catalog;
pub mod common;
pub mod macros;

pub mod routing;
mod store;

use std::collections::HashMap;
///This function is responsible for gathering and dispatching messages received from the
///macro invocation of `send!`. Multiple messages can be grouped for one more actors in
///one `send!` macro invocation as shown below:
///
///Example
///
///```
///use arrows::send;
///use arrows::Msg;
///
///let m1 = Msg::with_text("Message to actor1");
///let m2 = Msg::with_text("Message to actor1");
///let m3 = Msg::with_text("Message to actor2");
///let m4 = Msg::with_text("Message to actor1");
///let m5 = Msg::with_text("Message to actor1");
///send!("actor1", (m1, m2), "actor2", (m3), "actor1", (m4, m5));
///```
///Grouping within braces is not necessary while sending only to one actor:
///
///```
///let m6 = Msg::with_text("Message to actor3")
///let m7 = Msg::with_text("Message to actor3")
///send!("actor3",m6,m7);
///
///```
///Actors identified with string literal such as 'actor3' is assumed to be running in the
///local system(if they are not running - they would be resurrected).
///
///Actors running in remote systems - need to identified by the `Addr` construct:
///
///```
///use arrows::send;
///use arrows::Msg;
///use arrows::Addr;
///
///let remote_addr1 = Addr::remote("actor1", "10.10.10.10:7171");
///let remote_addr2 = Addr::remote("actor2", "11.11.11.11:8181");
///
///let m1 = Msg::with_text("Message to remote actor1");
///let m2 = Msg::with_text("Message to remote actor1");
///let m3 = Msg::with_text("Message to remote actor2");
///let m4 = Msg::with_text("Message to remote actor2");
///
///send!(remote_addr1, (m1,m2), remote_addr2, (m3,m4));
///
///```
///Messages for each actor will always be delivered in the order they are ingested into
///the system. Actor will not process out of sequence message. To prevent loss, messages
///are persisted into an embedded store backed by highly performant sqlite db.
///
///
///A new implementation of actor may be swapped in - replaching an actively running actor
///in the system. Swapped in actor would take over from an outgoing actor and start
///processing messages from where the outgoing left off. An actor would never process an
///out of sequence message i.e. it would never increment its message sequence counter until
///it has successfully processed the received message.
///
///Actors can change their behaviour while still running. They can create other actors
///copies of themselves.
///
///Actors are allowed to panic a set number of times(currently 3).
///
pub fn recv(msgs: HashMap<&Addr, Vec<Msg>>) {
    use crate::routing::messenger::Messenger;
    if let Err(err) = Messenger::send(msgs) {
        eprintln!("Error sending msgs {:?}", err);
    }
}

//A mail with extra details - inbound/outbound, seq, from & destined to
pub(crate) enum RichMail {
    Content(Mail, bool, i64, Option<Addr>, Option<Addr>),
}
use RichMail::Content;
impl RichMail {
    pub(crate) fn mail(&self) -> &Mail {
        let Content(mail, _, _, _, _) = self;
        mail
    }

    pub(crate) fn mail_out(&mut self) -> Mail {
        let Content(mail, _, _, _, _) = self;
        std::mem::replace(mail, Mail::Blank)
    }

    pub(crate) fn replace_mail(&mut self, msgs: Vec<Msg>) {
        let Content(mail, _, _, _, _) = self;
        *mail = Mail::Bulk(msgs);
    }

    pub(crate) fn to(&self) -> Option<&Addr> {
        let Content(_, _, _, _, to) = self;
        to.as_ref()
    }

    pub(crate) fn from(&self) -> Option<&Addr> {
        let Content(_, _, _, from, _) = self;
        from.as_ref()
    }

    pub(crate) fn inbound(&self) -> bool {
        let Content(_, inbound, _, _, _) = self;
        *inbound
    }

    pub(crate) fn seq(&self) -> i64 {
        let Content(_, _, seq, _, _) = self;
        *seq
    }
}
/***
 * Issues to be resolved and things to be done
 * 1. Documentation - absolute priority
 * 2. Test mode/Dev mode - routing
 * 3. Config
 * 4. Read documentation doc
 * 5. All examples should run
 * 6. Check first message panic
 * 7. Find logo and make it public
 * 8. store/apis clean up
 * ***/
