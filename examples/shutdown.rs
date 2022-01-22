use arrows::send;
use arrows::Addr;
use arrows::Msg;

fn main() {
    let _rs = send!(Addr::listen_addr(), Msg::shutdown());
}
