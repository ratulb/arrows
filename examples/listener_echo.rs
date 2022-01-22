use arrows::send;
use arrows::Msg;
use arrows::Addr;

fn main() {
    //Fire an echo message to the listener - it should reverse the string and send it back if
    //alive
    //let m = Msg::echo("This is an echo message to the listener");
    let m = Msg::echo("renetsil eht ot egassem ohce na si sihT");
    let _rs = send!(Addr::listen_addr(), m);
}
