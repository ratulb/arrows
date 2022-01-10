#![deny(rust_2018_idioms)]
pub use catalog::{restore, send_off};
pub use common::actor::{Actor, Producer, ProducerDeserializer};
pub use common::addr::Addr;
pub use common::errs::{Error, Result};
pub use common::mail::{Mail, Msg};
pub use common::utils::*;

pub(crate) use store::*;

pub mod catalog;
pub mod common;
pub mod macros;

mod routing;
mod store;

use std::collections::HashMap;
pub fn recv(msgs: HashMap<&Addr, Vec<Msg>>) {
    use crate::routing::Messenger;
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
