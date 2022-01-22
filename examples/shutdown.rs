use arrows::send;
use arrows::Addr;
use arrows::Msg;

fn main() {
    let rs = send!(Addr::listen_addr(), Msg::shutdown());
    println!("The send output = {:?}", rs);
}
