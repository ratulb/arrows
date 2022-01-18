use crate::catalog::ingress;
use crate::{from_bytes, Addr, Mail};
use byte_marks::Marked;
use std::io::{BufReader, BufWriter, Result, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

pub struct MessageListener {
    addr: SocketAddr,
}
impl MessageListener {
    pub(crate) fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
    pub fn start() {
        let listener_addr = Addr::new("listener");
        println!("Starting listener @{}", listener_addr);
        let listener =
            MessageListener::new(listener_addr.get_socket_addr().expect("Socket address"));
        let _rs = listener.run();
        println!("I am done running");
        println!("I am done running");
        println!("I am done running");
        println!("I am done running");
        println!("I am done running");
    }

    pub(crate) fn run(mut self) -> Result<()> {
        let listener = TcpListener::bind(self.addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(inner_stream) => match self.serve(inner_stream) {
                    Err(serving_error) => eprintln!("Error serving client {}", serving_error),
                    Ok(None) => continue,
                    Ok(cmd) => match cmd {
                        Some(cmd) if cmd.is_command() && cmd.command_is_same("stop") => {
                            println!("Stopping on request");
                            break;
                        }
                        _ => continue,
                    },
                },
                Err(e) => {
                    eprintln!("Error handling connection {}", e);
                }
            }
            println!("MessageListener served stream!");
        }
        Ok(())
    }

    fn serve(&mut self, tcp: TcpStream) -> Result<Option<Mail>> {
        let _peer_addr = tcp.peer_addr()?;
        let cloned = tcp.try_clone()?;
        let mut reader = BufReader::new(cloned);
        let mut writer = BufWriter::new(tcp);
        let marked = Marked::with_defaults(&mut reader);

        for mail in marked {
            println!("Received mail length = {:?}", mail);
            match self.ingress(mail) {
                Ok(cmd) if cmd.is_some() => return Ok(cmd),
                Ok(_) => continue,
                Err(err) => eprintln!("Error ingressing mail {:?}", err),
            }
        }
        writer.write_all("MessageListener received request".as_bytes())?;
        writer.flush()?;
        Ok(None)
    }
    fn process_cmd(mail: Mail) -> Result<Option<Mail>> {
        if mail.command_is_same("stop") {
            println!("The check is here1 and here!");
            println!("The check is here1 and here!");
            println!("The check is here1 and here!");
            println!("The check is here1 and here!");
            Ok(Some(mail))
        } else {
            Ok(None)
        }
    }

    fn ingress(&self, payload: Vec<u8>) -> Result<Option<Mail>> {
        let payload = from_bytes::<'_, Mail>(&payload)?;
        println!("Payload in listener ingress {:?}", payload);
        println!("Payload in listener ingress {:?}", payload);
        println!("Payload in listener ingress {:?}", payload.is_command());
        match payload {
            m @ Mail::Bulk(_) if m.is_command() && m.command_is_same("stop") => {
                println!("The check is here1");
                println!("The check is here1");
                println!("The check is here1");
                println!("The check is here1");
                println!("The check is here1");
                Self::process_cmd(m)
            }
            m @ Mail::Trade(_) | m @ Mail::Bulk(_) | m @ Mail::Blank => ingress(m),
            _ => {
                eprintln!("Sunk to blackhole!");
                return Ok(None);
            }
        };
        Ok(None)
    }
}
//use arrows::define_actor;
//use arrows::ExampleActorProducer;

/***fn define_example_actors() {
    let producer = ExampleActorProducer;
    let _rs = define_actor!("example_actor1", producer);
    let _rs = define_actor!("from", ExampleActorProducer);
    println!("Defined example actors");
}***/
