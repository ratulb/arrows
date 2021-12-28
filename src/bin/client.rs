use arrows::{option_of_bytes, Addr, Mail, Msg};
use byte_marks::ByteMarker;
use clap::AppSettings;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Result, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::process::exit;
use structopt::StructOpt;

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:7171";
const ADDRESS_FORMAT: &str = "IP:PORT";

#[derive(StructOpt, Debug)]
#[structopt(name="client",
            global_settings=&[AppSettings::DisableHelpSubcommand,
                              AppSettings::VersionlessSubcommands])]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "send", about = "Send text messages to an actor")]
    SendMessage {
        #[structopt(name = "ACTOR", help = "An actor's name")]
        actor: String,

        #[structopt(
            name = "MSG",
            help = "One(or more comma separated) message(s) to an actor"
        )]
        msg: String,

        #[structopt(
            long,
            help = "Sets the server address",
            value_name = ADDRESS_FORMAT,
            default_value = DEFAULT_LISTENING_ADDRESS,
            parse(try_from_str)
        )]
        addr: SocketAddr,
    },
}

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = run(opt) {
        eprintln!("{:?}", e);
        exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    match opt.command {
        Command::SendMessage { actor, msg, addr } => {
            let mut client = Client::connect(addr)?;
            client.send(&actor, &msg)?
        }
    }

    Ok(())
}

pub struct Client<'a> {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    marker: ByteMarker<'a>,
}

impl Client<'_> {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;
        let write_half = stream.try_clone()?;
        Ok(Client {
            reader: BufReader::new(stream),
            writer: BufWriter::new(write_half),
            marker: ByteMarker::with_defaults(),
        })
    }

    pub fn send(&mut self, actor: &str, msgs: &str) -> Result<()> {
        let msgs: Vec<_> = msgs
            .split(',')
            .map(|msg| Msg::new_with_text(msg, "cli", "actor"))
            .collect();
        let addr = Addr::new(actor);

        let bulk = Mail::Bulk(msgs);
        match option_of_bytes(&bulk) {
            Some(ref mut bytes) => {
                self.marker.mark_tail(bytes);
                self.writer.write_all(bytes)?;
                println!("Total bytes len = {:?}", bytes.len());
                self.writer.flush()?;
                let mut buf = vec![0; 1024];
                let len = self.reader.read(&mut buf)?;
                println!(
                    "Server response = {:?}",
                    String::from_utf8_lossy(&buf[..len])
                );
                Ok(())
            }
            None => {
                eprintln!("Error converting message to bytes");
                Err(Error::new(
                    ErrorKind::Other,
                    "Error converting message to bytes",
                ))
            }
        }
    }
}
