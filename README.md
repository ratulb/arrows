# Arrows
###### An actor framework in rust with message durability and ingestion order processing of of messages. Message persistence via an embedded sqlite instance. Message content can be text or binary(Vec<u8>). Messages themselves get stored as binany in the backing store.

```rust
use crate::{Actor, Mail, Msg, Producer};
use serde::{Deserialize, Serialize};

//A sample actor
pub struct DemoActor;
impl Actor for DemoActor {
    fn receive(&mut self, incoming: Mail) -> Option<Mail> {
        match incoming {
            Mail::Trade(msg) => println!("Received: {}", msg),
            bulk @ Mail::Bulk(_) => println!("Received bulk msg: {}", bulk),
            Mail::Blank => println!("DemoActor received blank"),
        }
        Some(Msg::from_text("Message from DemoActor").into())
    }
}

//Producer implementations are called to produce actor instances.

//Produces DemoActor instances
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ActorProducer;

//Producer implementations need to be tagged with `typetag` marker.

#[typetag::serde]
impl Producer for ActorProducer {
    fn produce(&mut self) -> Box<dyn Actor> {
        Box::new(DemoActor)
    }
}

//The `define_actor` - macro actually defines a new actor `instance` in the system. The
//actor instance along with the producer - get persisted in the backing store, the actor
//instance gets activated and receives a startup signal and becomes ready to process
//incoming messages. The producer defintion gets used to restart/restore the actor as
//required.

use arrows::define_actor;

let producer = ActorProducer::default();
define_actor!("demo_actor", producer);

//At this stage - the actor instance `demo_actor` is ready for incoming messages. It
//should have already received the startup signal.

use arrows::send;

let m1 = Msg::from_text("Message to demo actor");
let m2 = Msg::from_text("Message to demo actor");
let m3 = Msg::from_text("Message to demo actor");

send!("demo_actor", (m1, m2, m3));

//Create another actor instance - demo_actor1

define_actor!("demo_actor1", ActorProducer::default());

let m4 = Msg::from_text("Message to demo actor1");
let m5 = Msg::from_text("Message to demo actor1");

let m6 = Msg::from_text("Message to demo actor");
let m7 = Msg::from_text("Message to demo actor");

//Send out multiple messages to multiple actors at one go

send!("demo_actor1", (m4, m5), "demo_actor", (m6, m7));


//Actors running in remote systems - need to be identified by the `Addr` construct:

use arrows::Addr;

let remote_actor = Addr::remote("remote_actor", "11.11.11.11:8181");

let m1 = Msg::with_text("Message to remote actor");
let m2 = Msg::with_text("Message to remote actor");

send!("remote_actor", m1, m2);

//While sending to a single actor - its not recessary to group messages within braces.
```

##### How to get started:

* Check this repository out
* Launch an extra terminal
* Fire the `register.sh` script in the project directory.
* In another terminal launch the `server.sh` script in the same directory
* From previous termainal launch the `send.sh` script - actors should start receiving messages


##### Contribution: This project is still evolving. Contributions are welcome.
