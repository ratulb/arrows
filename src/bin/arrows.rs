use arrows::common::config::Config;
use arrows::routing::listener::MessageListener;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "arrows")]
struct Opt {
    #[structopt(
        long,
        short = "i",
        name = "hostport",
        help = "Flag to overide default listen address",
        parse(try_from_str)
    )]
    hostport: Option<String>,

    #[structopt(
        long,
        short = "d",
        name = "db",
        help = "Specify backing store path",
        parse(try_from_str)
    )]
    db: Option<String>,

    #[structopt(
        long,
        help = "Set the listening address",
        value_name = "IP:PORT",
        parse(try_from_str),
        required_if("hostport", "user")
    )]
    addr: Option<SocketAddr>,
}
//cargo run --bin arrows -- -i user --addr 127.0.0.1:8181
//cargo run --bin arrows -- -i user --addr 127.0.0.1:8181 -d /tmp
//cargo run --bin arrows -- -i user --addr 127.99.1.1:8182 -d /tmp

fn main() {
    let opts = Opt::from_args();
    let mut config = Config::from_env();
    match opts.hostport {
        None => return MessageListener::start(),
        Some(ref hostport) if hostport == "user" => match opts.addr {
            Some(ref sa) => {
                match sa.ip() {
                    IpAddr::V4(inner) => {
                        let host = inner.to_string();
                        config.set_host(&host);
                    }
                    _ => eprintln!("Ipv6Addr address not supported currently! "),
                }
                let port = sa.port();
                config.set_port(port);
            }
            None => panic!("IP:PORT expected!"),
        },
        _ => eprintln!("Wrong options!"),
    };

    if let Some(dbpath) = opts.db {
        if !Path::new(&dbpath).exists() {
            panic!("Db path does not exits {}!", dbpath);
        } else {
            config.set_db_path(&dbpath);
        }
    }

    Config::re_init(config);
    MessageListener::start();
}
