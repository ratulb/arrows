use arrows::define_actor;
use arrows::send;
use arrows::Addr;
use arrows::AnotherProducer;
use arrows::Msg;
use arrows::NewProducer;

fn main() {
    let actor_producer = NewProducer::default();
    /***    define_actor!("new_actor", actor_producer);

    let another_producer = AnotherProducer::default();
    define_actor!("another_actor", another_producer);***/

    /***let m1 = Msg::from_text("Message to new_actor");
        let m2 = Msg::from_text("Message to new_actor");
        let m3 = Msg::from_text("Message to new_actor");
        send!("new_actor", (m1, m2, m3));
    ***/
    let mut m4 = Msg::from_text("Message to another_actor");

    //Impersonate sender as "new_actor"
    m4.set_from(&Addr::new("new_actor"));

    send!("another_actor", m4);
}
