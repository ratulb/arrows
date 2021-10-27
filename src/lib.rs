#![deny(rust_2018_idioms)]
pub(crate) use crate::boxes::STORE;
pub use crate::boxes::*;
pub use crate::etc::*;
pub use arrow_commons::Actor;
pub use arrow_commons::Addr;
pub use arrow_commons::Message;

pub mod actors;
pub mod boxes;
mod storage;

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
