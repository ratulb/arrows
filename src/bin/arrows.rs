use arrows::from_bytes;
use arrows::type_of;
use arrows::Mail;
use byte_marks::Marks;
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
            println!("Server served stream!");
        }
        Ok(())
    }

    fn serve(&mut self, tcp: TcpStream) -> Result<()> {
        let peer_addr = tcp.peer_addr()?;
        let mut reader = BufReader::new(tcp.try_clone()?);
        let mut writer = BufWriter::new(tcp);
        let mut message_bytes = Unmarkable::new(&mut reader);
        println!("Connection from = {:?}", peer_addr);

        /***let buf = reader.fill_buf()?;
        println!("Buf len = {:?}", buf.len());

        let unmarked = Marks::unmark(buf);

        if let Some(unmarked) = unmarked {
            println!("Len is = {:?}", unmarked.0.len());
            let mail: Mail = from_bytes(unmarked.0[0]).unwrap();
            println!("Msg is = {:?}", mail);
        }***/

        while let Some(inner) = message_bytes.next() {
            type_of(&inner);

            println!("Mgs len here");
        }
        writer.write_all("Server received request".as_bytes())?;
        writer.flush()?;
        println!("Server reached then here!");
        Ok(())
    }
}
