//!Ingress happens via the `MessegeListener` struct defined in this module
//!The listener binary can can be manually triggered via `cargo run --bin arrows`.
//!
//!It would be automatically launched when a message ingress happens via the `send!`
//!macro invocation
//!
use crate::catalog::ingress;

use crate::{from_bytes, Action, Addr, Config, Mail, Mail::Bulk};
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
                    Ok(None) => continue,
                    Ok(cmd) => match cmd {
                        Some(_) => {
                            println!("Stopping on request");
                            break;
                        }
                        None => continue,
                    },
                },
                Err(e) => {
                    eprintln!("Error handling connection {}", e);
                }
            }
        }
        Ok(())
    }

    fn serve(&mut self, tcp: TcpStream) -> Result<Option<Mail>> {
        let cloned = tcp.try_clone()?;
        let mut reader = BufReader::new(cloned);
        let mut writer = BufWriter::new(tcp);
        let marked = Marked::with_defaults(&mut reader);
        let mut response = String::from("MessageListener received request");
        for mail in marked {
            match self.process(mail) {
                Ok(Some(mail)) => match mail {
                    mail @ Bulk(_) if mail.command_equals(Action::Shutdown) => {
                        return Ok(Some(mail));
                    }
                    mail @ Bulk(_) if mail.command_equals(Action::Echo("".to_string())) => {
                        response.clear();
                        let _rs = match mail.get_action() {
                            Some(mut action) => action.execute(mail),
                            None => None,
                        };

                        /***if let Some(text) = mail.messages()[0].as_text() {
                            println!("The echo text {}", text);
                            response.push_str(&text.chars().rev().collect::<String>())
                        }***/
                        break;
                    }
                    _ => continue,
                },
                Ok(None) => continue,
                Err(err) => eprintln!("Error ingressing mail {}", err),
            }
        }
        writer.write_all(response.as_bytes())?;
        writer.flush()?;
        Ok(None)
    }

    fn process(&self, payload: Vec<u8>) -> Result<Option<Mail>> {
        let payload = from_bytes::<'_, Mail>(&payload)?;
        match payload {
            m @ Mail::Bulk(_) if m.is_command() => Ok(Some(m)),
            m @ Mail::Bulk(_) => ingress(m),
            _ => Ok(None),
        }
    }
}
