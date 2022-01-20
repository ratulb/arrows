use arrows::send;

use arrows::Msg;

pub fn main() {
    /***let m1 = Msg::with_text("Message to actor1");
    let m2 = Msg::with_text("Message to actor1");
    let m3 = Msg::with_text("Message to actor2");
    let m4 = Msg::with_text("Message to actor1");
    let m5 = Msg::with_text("Message to actor1");
    send!("actor1", (m1, m2), "actor2", (m3), "actor1", (m4, m5));***/

    send!(
        "actor3",
        Msg::with_text("Message to actor3"),
        Msg::with_text("Message to actor3")
    );
    /***send!("actor3", Mail::Blank, Mail::Blank);
    send!("actor3", Mail::Blank, Mail::Blank);
    send!("actor3", Mail::Blank, Mail::Blank);***/
}
