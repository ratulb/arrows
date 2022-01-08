#![deny(rust_2018_idioms)]
pub use catalog::{restore, send_off};
pub use common::actor::{Actor, ActorBuilder, BuilderDeserializer};
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

pub(crate) type DetailedMsg = (Msg, bool, i64);

use std::collections::HashMap;
pub fn recv(msgs: HashMap<&Addr, Vec<Msg>>) {
    use crate::routing::Messenger;
    Messenger::send(msgs);
}
