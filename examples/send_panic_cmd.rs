use arrows::send;
use arrows::Msg;

fn main() {
    //Sends to example actor defined in src/common/actors.rs example_actor1
    let panic_cmd = Msg::command_from("stop");
    let rs = send!("example_actor1", panic_cmd);
    println!("The send output = {:?}", rs);
}
