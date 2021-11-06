#![deny(rust_2018_idioms)]
pub use actor::{Actor, FakeActorBuilder};
pub use addr::Addr;
pub use errs::{Error, Result};
pub use msg::Msg;
pub use utils::*;
pub mod actor;
pub mod addr;
mod errs;
pub mod msg;
pub mod utils;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
