use arrows::send;
use arrows::Addr;
use arrows::Msg;

fn main() {
    //Fire an echo message to the listener - it should reverse the string and send it back if
    //alive
    //let m = Msg::echo("This is an echo message back from the listener");
    let m = Msg::echo("renetsil eht morf kcab egassem ohce na si sihT");
    send!(Addr::listen_addr(), m);
}
