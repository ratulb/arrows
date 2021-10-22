pub use crate::actor::Actor;
pub use crate::address::Address;
pub(crate) use crate::boxes::STORE;
pub use crate::boxes::*;
pub use crate::message::*;
pub use crate::utils::*;

pub use crate::etc::*;

pub mod actor;
pub mod actors;
pub mod address;
pub mod boxes;
pub mod message;
mod storage;
pub mod utils;

pub mod etc;

pub async fn start() {
    use crate::actors::REQUEST_VALIDATOR;
    println!("System startup check1");
    actors::start();
    for _ in 0..3 {
        let welcome = Message::internal(None, "actor-invoker", REQUEST_VALIDATOR);
        actors::ActorInvoker::invoke(welcome);
    }
}
