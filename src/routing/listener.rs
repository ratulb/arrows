use crate::catalog::ingress;

use crate::{from_bytes, Action, Addr, Mail, Mail::Bulk};
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
        println!("Starting listener @{}", listener_addr);
        let listener =
            MessageListener::new(listener_addr.get_socket_addr().expect("Socket address"));
        let _rs = listener.run();
        println!("Listener exiting...");
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
            match self.ingress(mail) {
                Ok(Some(mail)) => match mail {
                    mail @ Bulk(_) if mail.command_equals(Action::Shutdown) => {
                        return Ok(Some(mail))
                    }
                    mail @ Bulk(_) if mail.command_equals(Action::Echo("".to_string())) => {
                        response.clear();
                        if let Some(text) = mail.messages()[0].as_text() {
                            response.push_str(text)
                        }
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

    fn ingress(&self, payload: Vec<u8>) -> Result<Option<Mail>> {
        let payload = from_bytes::<'_, Mail>(&payload)?;
        match payload {
            m @ Mail::Bulk(_) if m.is_command() => Ok(Some(m)),
            m @ Mail::Trade(_) | m @ Mail::Bulk(_) => ingress(m),
            _ => Ok(None),
        }
    }
}
