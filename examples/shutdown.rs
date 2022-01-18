use arrows::send;
use arrows::Msg;

fn main() {
    //Sends to example actor defined in src/common/actors.rs example_actor1
    let rs = send!("example_actor1", Msg::shutdown());
    println!("The send output = {:?}", rs);
}
