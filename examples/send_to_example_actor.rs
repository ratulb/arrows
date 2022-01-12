use arrows::send;
use arrows::Msg;

fn main() {
    //Sends to example actor defined in src/common/actors.rs example_actor1
    //let m = Msg::default();Sending this multiple times would lead to constraint violation
    let m = Msg::new_with_text("mail", "from", "to3");
    let rs  =send!("example_actor1", m);
    println!("The send output = {:?}", rs);
}
