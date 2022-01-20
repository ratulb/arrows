use arrows::send;
use arrows::Addr;
use arrows::Msg;

pub fn main() {
    let m1 = Msg::from_text("Message to actor1");
    let m2 = Msg::from_text("Message to actor1");
    let m3 = Msg::from_text("Message to actor2");
    let m4 = Msg::from_text("Message to actor1");
    let m5 = Msg::from_text("Message to actor1");
    send!("actor1", (m1, m2), "actor2", (m3), "actor1", (m4, m5));

    let remote_addr1 = Addr::remote("actor1", "10.10.10.10:7171");
    let remote_addr2 = Addr::remote("actor2", "11.11.11.11:8181");

    let m1 = Msg::from_text("Message to remote actor1");
    let m2 = Msg::from_text("Message to remote actor1");
    let m3 = Msg::from_text("Message to remote actor2");
    let m4 = Msg::from_text("Message to remote actor2");

    send!(remote_addr1, (m1, m2), remote_addr2, (m3, m4));
}
