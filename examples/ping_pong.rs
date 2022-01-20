use arrows::send;
use arrows::Msg;

fn main() {
    let m = Msg::with_text("mail", "from", "example_actor1");
    let rs = send!("example_actor1", m);
    println!("The send output = {:?}", rs);
}
