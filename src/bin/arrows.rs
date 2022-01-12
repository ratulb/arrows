use arrows::ingress;
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
    define_example_actors();
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
        let _peer_addr = tcp.peer_addr()?;
        let cloned = tcp.try_clone()?;
        let mut reader = BufReader::new(cloned);
        let mut writer = BufWriter::new(tcp);
        let marked = Marked::with_defaults(&mut reader);

        for mail in marked {
            println!("Received mail length = {}", mail.len());
            let rs = self.ingress(mail);
            if let Err(err) = rs {
                eprintln!("Error ingressing mail {:?}", err);
            }
        }
        writer.write_all("Server received request".as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    fn ingress(&self, payload: Vec<u8>) -> Result<()> {
        let payload = from_bytes::<'_, Mail>(&payload)?;
        match payload {
            m @ Mail::Trade(_) | m @ Mail::Bulk(_) | m @ Mail::Blank => ingress(m),
            _ => eprintln!("Engulfed by blackhole!"),
        }
        Ok(())
    }
}
use arrows::define_actor;
use arrows::Addr;
use arrows::ExampleActorProducer;

fn define_example_actors() {
    let producer = ExampleActorProducer;
    let _rs = define_actor!("example_actor1", producer);
    println!("Defined example actors");
}
