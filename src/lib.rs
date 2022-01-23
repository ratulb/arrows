#![deny(rust_2018_idioms)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub(crate) use common::actor::ProducerDeserializer;
pub use common::actor::{Actor, Producer};
pub use common::addr::Addr;
pub use common::config::Config;
pub use common::errs::{Error, Result};
pub(crate) use common::mail::RichMail;
pub use common::mail::{Action, Mail, Msg};
pub use common::utils::*;
pub use demos::*;
pub(crate) use store::*;

pub mod catalog;
pub mod common;
pub mod macros;

pub mod demos;
pub mod routing;
mod store;
