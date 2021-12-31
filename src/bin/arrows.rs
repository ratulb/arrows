use arrows::persist_mail;
use arrows::{from_bytes, Mail};
use byte_marks::Marked;
use std::io::{BufReader, BufWriter, Result, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use structopt::StructOpt;

const DEFAULT_LISTENING_ADDRESS: &str = "0.0.0.0:7171";

#[derive(StructOpt, Debug)]
#[structopt(name = "server")]
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
    let server = Server::default();
    server.run(opt.addr);
    Ok(())
}

#[derive(Default)]
pub struct Server;

impl Server {
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
            println!("Server served stream!");
        }
        Ok(())
    }

    fn serve(&mut self, tcp: TcpStream) -> Result<()> {
        let peer_addr = tcp.peer_addr()?;
        let cloned = tcp.try_clone()?;
        let mut reader = BufReader::new(cloned);
        let mut writer = BufWriter::new(tcp);
        let marked = Marked::with_defaults(&mut reader);

        for mail in marked {
            println!("Received mail length = {}", mail.len());
            self.route_mail(mail);
        }
        writer.write_all("Server received request".as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    fn route_mail(&self, mail: Vec<u8>) -> Result<()> {
        let mail = from_bytes::<'_, Mail>(&mail)?;
        match mail {
            trade @ Mail::Trade(_) => persist_mail(trade),
            bulk @ Mail::Bulk(_) => {
                persist_mail(bulk);
            }
            Mail::Blank => eprintln!("Blank"),
        }
        Ok(())
    }
}
