#![deny(rust_2018_idioms)]
pub use actor::{Actor, ActorBuilder, FakeActorBuilder};
pub use addr::Addr;
pub use errs::{Error, Result};
pub use msg::Msg;
pub use utils::*;
mod actor;
mod addr;
mod errs;
mod msg;
mod utils;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
