# arrows
##### Arrows - a fast, lightweight, resilient actor framework in rust. 

##### Current supported functionalities:

* Message durability is intrinsic(Can not be opted out). Based on fast sqlite embedded instance. (https://github.com/rusqlite/rusqlite)
* Remoting(No peer awareness - other systems should be up or remote delivery fails and gets retried at system start up) - as of now.
* Binany(serde https://github.com/serde-rs/serde + bincode https://github.com/bincode-org/bincode) or Text payload
* Actor panic toleration.
* Runtime swapping of actor behaviour with another actor definition(Actor binaries has to be available in system - no runtime injection of binaries)
* No out of sequence delivery of messages 
* Swapped in actor resumes from where the swapped out instance left off.
* Actor loading/Unloading is based on typetag(https://github.com/dtolnay/typetag).
* Multiple instances of the same actor - with different named identifier
* Macro for defining actor(`define_actor!`)
* Macro for sending message(s) to actor(s) - (`send!`)
* Panicking Actor ejection.
* Parralel processing of received messages 
* Post start and clean up signals
* No boot up required. Post an echo msg and the server would be ready.

```rust
 use crate::{define_actor, send};
 use serde::{Deserialize, Serialize};
 
 //Sample actor and actor producer
pub struct ExampleActor;
impl Actor for ExampleActor {
    fn receive(&mut self, incoming: Mail) -> Option<Mail> {
        println!("Actor received {}", incoming.message());
        None
    }
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ExampleActorProducer;
impl Producer for ExampleActorProducer {
    fn produce(&mut self) -> Box<dyn Actor> {
        Box::new(ExampleActor)
    }
}
fn main() {
  let producer = ExampleActorProducer;
  //Define an actor
  let _rs = define_actor!("example_actor1", producer);
  let m = Msg::default();
  //Send out messages
  send!("example_actor1", m);
}
 ```
