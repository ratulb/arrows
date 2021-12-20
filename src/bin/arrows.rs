use byte_marks::Unmarkable;
use std::fs;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Result, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use structopt::StructOpt;

const DEFAULT_LISTENING_ADDRESS: &str = "0.0.0.0:7171";

#[derive(StructOpt, Debug)]
#[structopt(name = "arrow-server")]
struct Opt {
    #[structopt(
        long,
        help="Set the listening address",
        value_name="IP:PORT",
        default_value =DEFAULT_LISTENING_ADDRESS,
        parse(try_from_str),
        )]
    addr: SocketAddr,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    println!("Server listening on {}", opt.addr);
    let server = ArrowServer::default();
    server.run(opt.addr);
    Ok(())
}

#[derive(Default)]
pub struct ArrowServer;

impl ArrowServer {
    pub fn run<A: ToSocketAddrs>(mut self, addr: A) -> Result<()> {
        let listener = TcpListener::bind(addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(inner_stream) => {
                    if let Err(serving_error) = self.serve(inner_stream) {
                        eprintln!("Error serving client {:?}", serving_error);
                    }
                }
                Err(e) => {
                    eprintln!("Error handling connection {:?}", e);
                }
            }
        }
        Ok(())
    }

    fn serve(&mut self, tcp: TcpStream) -> Result<()> {
        let peer_addr = tcp.peer_addr()?;
        let mut reader = BufReader::new(&tcp);
        let mut writer = BufWriter::new(&tcp);
        let message_bytes = Unmarkable::new(&mut reader);

        /***macro_rules! do_reply {
            ($reply:expr) => {{
                let reply = $reply;
                serde_json::to_writer(&mut writer, &reply)?;
                writer.flush()?;
                println!("Reply sent to {:?} -> {:?}", peer_addr, reply);
            }};
        }***/

        for bytes in message_bytes {
            /***let request = request?;
            println!("Request received from {:?} -> {:?}", peer_addr, request);

            match request {
                Request::Get { key } => do_reply!(match self.engine.get(key) {
                    Ok(value) => GetResponse::Ok(value),
                    Err(e) => GetResponse::Err(e.to_string()),
                }),
                Request::Remove { key } => do_reply!(match self.engine.remove(key) {
                    Ok(_) => RemoveResponse::Ok(()),
                    Err(e) => RemoveResponse::Err(e.to_string()),
                }),
                Request::Set { key, value } => do_reply!(match self.engine.set(key, value) {
                    Ok(_) => SetResponse::Ok(()),
                    Err(e) => SetResponse::Err(e.to_string()),
                }),
            };***/
        }
        Ok(())
    }
}
