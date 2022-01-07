#![deny(rust_2018_idioms)]
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
mod sender;
mod store;

pub(crate) type DetailedMsg = (Msg, bool, i64);
