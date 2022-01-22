use arrows::send;
use arrows::Msg;

fn main() {
    arrows::demos::register();
    let m1 = Msg::from_text("Message to new_actor");
    let m2 = Msg::from_text("Message to new_actor");
    let m3 = Msg::from_text("Message to new_actor");
    //Send messages
    send!("new_actor", (m1, m2, m3));
}
