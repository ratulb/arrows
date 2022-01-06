#![deny(rust_2018_idioms)]
pub use common::actor::{Actor, ActorBuilder, BuilderDeserializer};
pub use common::addr::Addr;
pub use common::errs::{Error, Result};
pub use common::mail::Mail;
pub use common::mail::Msg;
pub use common::utils::*;
pub use registry::persist_mail;
pub(crate) use store::*;

pub mod common;
pub mod macros;
pub mod registry;

mod routing;
mod sender;
mod store;

pub(crate) type DetailedMsg = (Msg, bool, i64);
