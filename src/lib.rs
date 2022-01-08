#![deny(rust_2018_idioms)]
pub use catalog::restore;
pub use catalog::send_mail;
pub use common::actor::{Actor, ActorBuilder, BuilderDeserializer};
pub use common::addr::Addr;
pub use common::errs::{Error, Result};
pub use common::mail::Mail;
pub use common::mail::Msg;
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
    for (k, v) in msgs.iter() {
        println!("Key({:?}) and msg count({:?})", k.get_name(), v.len());
    }
}
