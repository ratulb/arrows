use arrows::send;
use arrows::Msg;

fn main() {
    let m = Msg::echo("This is an echo message to the listener");
    let rs = send!("example_actor1", m);
    println!("The send output = {:?}", rs);
}
