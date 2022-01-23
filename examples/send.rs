use arrows::send;
use arrows::{Addr, Msg};

fn main() {
    let m1 = Msg::from_text("Message to new_actor");
    let m2 = Msg::from_text("Message to new_actor");
    let m3 = Msg::from_text("Message to new_actor");

    send!("demo_actor", (m1, m2, m3));

    let mut m4 = Msg::from_text("Message to another_actor");

    //Impersonate sender as "new_actor"
    m4.set_from(&Addr::new("demo_actor"));

    send!("another_actor", m4);
}
