//!Ingress happens via the `MessegeListener` struct defined in this module
//!The listener binary can can be manually triggered via `cargo run --bin arrows`.
//!
//!It would be automatically launched when a message ingress happens via the `send!`
//!macro invocation
//!
use crate::catalog::ingress;

use crate::{from_bytes, Action, Action::*, Addr, Config, Mail};
use byte_marks::Marked;
use std::io::{BufReader, BufWriter, Result, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

///Message ingestion entry point of the actor system. Each listener instance fronts a
///completely independent actor system that supports message persistence, actor life-cycle
///management(defintion, activation/passivation, panicking actor eviction etc) and remoting.

pub struct MessageListener {
    addr: SocketAddr,
}
impl MessageListener {
    pub(crate) fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
    ///Starts up the message ingester binary on a given node. Multiple listeners can be
    ///running on a given node as long as their ports do not conflict and respective
    ///backing stores point to different locations(configurable via the environment
    ///variable `DB_PATH`) in the node.
    ///
    pub fn start() {
        let listener_addr = Addr::new("listener");
        println!("Starting listener with config {:?}", Config::get_shared());
        let listener =
            MessageListener::new(listener_addr.get_socket_addr().expect("Socket address"));
        match listener.run() {
            Ok(_) => println!("Listener exiting..."),
            Err(err) => println!("Listener failed to run: {}", err),
        }
    }

    pub(crate) fn run(mut self) -> Result<()> {
        let listener = TcpListener::bind(self.addr)?;
        for stream in listener.incoming() {
            match stream {
                Ok(inner_stream) => match self.serve(inner_stream) {
                    Err(serving_error) => eprintln!("Error serving client {}", serving_error),
                    Ok(Shutdown) => {
                        println!("Shutdown request received!");
                        break;
                    }
                    Ok(_) => continue,
                },
                Err(e) => {
                    eprintln!("Error handling connection {}", e);
                }
            }
        }
        Ok(())
    }

    fn serve(&mut self, tcp: TcpStream) -> Result<Action> {
        let cloned = tcp.try_clone()?;
        let mut reader = BufReader::new(cloned);
        let mut writer = BufWriter::new(tcp);
        let marked = Marked::with_defaults(&mut reader);
        let mut response = String::from("Ok");
        for mail in marked {
            match self.process(mail) {
                Ok(Shutdown) => return Ok(Shutdown),
                Ok(Continue) => continue,
                Ok(Echo(text)) => {
                    response.clear();
                    response.push_str(&text);
                    break;
                }
                Err(err) => eprintln!("Error ingressing mail {}", err),
            }
        }
        writer.write_all(response.as_bytes())?;
        writer.flush()?;
        Ok(Continue)
    }
    //Process action from mails - that might contain command
    fn process_cmd(cmd: Mail) -> Result<Action> {
        match cmd.action() {
            Some(Shutdown) => Ok(Shutdown),
            Some(Continue) => Ok(Continue),
            //Take out the echo action from mail, execute it feeding the mail, return the
            //echo action from the ouput of the execution
            //Executing echo action reverses the string inside it
            Some(mut echo @ Echo(_)) => Ok(echo
                .execute(cmd)
                .map_or(Continue, |mail| mail.action().unwrap_or(Continue))),
            None => Ok(Continue),
        }
    }

    fn process(&self, payload: Vec<u8>) -> Result<Action> {
        let payload = from_bytes::<'_, Mail>(&payload)?;
        match payload {
            m @ Mail::Bulk(_) if m.is_command() => Self::process_cmd(m),
            m @ Mail::Bulk(_) => ingress(m).map(|_| Continue),
            _ => Ok(Continue),
        }
    }
}
