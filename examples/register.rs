use arrows::define_actor;

use arrows::{AnotherProducer, DemoActorProducer};

fn main() {
    let producer = DemoActorProducer::default();
    //Define actor instance with a producer instance
    define_actor!("demo_actor", producer);

    //Another actor producer combination
    let another_producer = AnotherProducer::default();
    define_actor!("another_actor", another_producer);

    //Create another actor instance from same producer defintion
    let producer = AnotherProducer::default();
    define_actor!("yet_another_actor", producer);
}
