use arrows::send;
use arrows::Addr;
use arrows::Msg;

fn main() {
    //Sends to example actor defined in src/common/actors.rs example_actor1
    let rs = send!(Addr::new("some"), Msg::shutdown());
    println!("The send output = {:?}", rs);
}
