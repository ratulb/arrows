use arrows::send;
use arrows::Msg;

fn main() {
    let m = Msg::from_text("Message for new actor");
    send!("new_actor", m);
}
