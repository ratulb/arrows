#![deny(rust_2018_idioms)]
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
pub fn recv(msgs: HashMap<&Addr, Vec<Msg>>) {
    use crate::routing::messenger::Messenger;
    Messenger::send(msgs);
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
 ***/

/***use std::sync::Once;
use std::thread;

static INIT: Once = Once::new();

assert_eq!(INIT.is_completed(), false);
let handle = thread::spawn(|| {
    INIT.call_once(|| panic!());
});
assert!(handle.join().is_err());
assert_eq!(INIT.is_completed(), false);***/
