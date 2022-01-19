use crate::catalog::ingress;

use crate::type_of;
use crate::{from_bytes, Action, Addr, Mail, Mail::Bulk};
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
        println!("In listener post run");
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
        let mut response = String::from("MessageListener received request");
        for mail in marked {
            println!("for mail in marked");
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
    fn process_cmd(mail: Mail) -> Result<Option<Mail>> {
        let what = Ok(Some(mail));
        println!("Returned process_cmd - {:?}!", what);
        what
    }

    fn ingress(&self, payload: Vec<u8>) -> Result<Option<Mail>> {
        let payload = from_bytes::<'_, Mail>(&payload)?;
        println!("The type of check in listener self ingress");
        type_of(&payload);
        match payload {
            m @ Mail::Bulk(_) if m.is_command() => {
                println!("The match payload going go to process_cmd");
                return Self::process_cmd(m);
            }
            m @ Mail::Trade(_) | m @ Mail::Bulk(_) => {
                println!("The match payload going to ingress in to db {}", m);
                ingress(m)
            }
            _ => {
                eprintln!("Sunk to blackhole!");
                return Ok(None);
            }
        };
        Ok(None)
    }
}
