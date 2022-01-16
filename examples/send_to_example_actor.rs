use arrows::send;
use arrows::Msg;

fn main() {
    //Sends to example actor defined in src/common/actors.rs example_actor1
    let m = Msg::new_with_text("mail", "from", "example_actor1");
    let rs = send!("example_actor1", m);
    println!("The send output = {:?}", rs);
}
