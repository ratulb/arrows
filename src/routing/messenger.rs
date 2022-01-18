use crate::routing::messenger::client::Client;
use crate::Error;
use crate::{Addr, Mail, Msg, Result};
use std::collections::HashMap;
use std::io::ErrorKind::ConnectionRefused;
use std::net::SocketAddr;
use std::process::Command;
pub(crate) struct Messenger;

impl Messenger {
    pub(crate) fn send(mails: HashMap<&Addr, Vec<Msg>>) -> Result<()> {
        mails.into_iter().for_each(|(addr, mut msgs)| {
            msgs.iter_mut().map(|msg| {
                msg.set_recipient_addr(addr);
                msg
            });
            if let Some(host_addr) = addr.get_socket_addr() {
                match Client::connect(host_addr) {
                    Ok(mut client) => {
                        println!("Messenger sent to *****{:?}", msgs);
                        println!("Messenger sent to *****{:?}", msgs);
                        println!("Messenger sent to *****{:?}", msgs);
                        println!("Messenger sent to *****{:?}", msgs);
                        println!("Messenger sent to *****{:?}", msgs);
                        println!("Messenger sent to *****{:?}", msgs);
                        client.send(msgs);
                        println!("Messenger sent to {}", host_addr);
                    }
                    Err(err) => {
                        eprintln!("Host: {} {}", host_addr, err);
                        try_restart_listener(msgs, host_addr, err);
                    }
                }
            }
        });
        Ok(())
    }

    pub(crate) fn mail(mail: Mail) -> Result<()> {
        Self::group_by(mail.take_all())
            .into_iter()
            .for_each(|(host_addr, msgs)| {
                match Client::connect(host_addr) {
                    Ok(mut client) => {
                        client.send(msgs);
                        println!("Messages sent to {}", host_addr);
                        Ok(())
                    }
                    Err(err) => {
                        eprintln!("Host: {} {}", host_addr, err);
                        try_restart_listener(msgs, host_addr, err)
                    }
                };
            });
        Ok(())
    }

    fn group_by(msgs: Vec<Msg>) -> HashMap<SocketAddr, Vec<Msg>> {
        let mut groups: HashMap<SocketAddr, Vec<Msg>> = HashMap::new();
        for msg in msgs {
            let host_addr = match msg.get_to() {
                Some(ref addr) => addr.get_socket_addr().expect("host"),
                None => panic!(),
            };
            groups.entry(host_addr).or_default().push(msg);
        }
        groups
    }
}
pub(super) mod client {

    use crate::{option_of_bytes, Mail, Msg};
    use byte_marks::ByteMarker;

    use std::io::{BufReader, BufWriter, Error, ErrorKind, Read, Result, Write};
    use std::net::{TcpStream, ToSocketAddrs};

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

        pub fn send(&mut self, msgs: Vec<Msg>) -> Result<()> {
            let bulk = Mail::Bulk(msgs);
            match option_of_bytes(&bulk) {
                Some(ref mut bytes) => {
                    self.marker.mark_tail(bytes);
                    self.writer.write_all(bytes)?;
                    self.writer.flush()?;
                    let mut buf = vec![0; 256];
                    let len = self.reader.read(&mut buf)?;
                    println!("From Server = {:?}", String::from_utf8_lossy(&buf[..len]));
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
}

fn try_restart_listener(
    _msgs: Vec<Msg>,
    socket_addr: SocketAddr,
    err: std::io::Error,
) -> Result<()> {
    
    let should_start =  std::env::var("SHOULD_START").unwrap_or_else(|_| "".to_string());
    if should_start == "".to_string() {
      println!("Should start should be empty and not starting: {}", should_start);
      println!("Should start should be empty and not starting: {}", should_start);
      println!("Should start should be empty and not starting: {}", should_start);
      println!("Should start should be empty and not starting: {}", should_start);
      return Ok(());
    }
    else {

      println!("Should start should not be empty and starting: {}", should_start);
      println!("Should start should not be empty and starting: {}", should_start);
      println!("Should start should not be empty and starting: {}", should_start);
      println!("Should start should not be empty and starting: {}", should_start);
    }

    if socket_addr.ip().is_loopback() {
        match err.kind() {
            ConnectionRefused => {
                println!("The error kind: {:?}", err.kind());
                /*** Command::new("sh")
                .spawn()
                .expect("sh command failed to start");***/
                println!("Current dir: {:?}", std::env::current_dir());
                let mut listener_path = std::env::current_dir().unwrap();
                //let listener_path = listener_path.push("/target/debug/arrows");
                listener_path.push("target/debug/arrows");
                println!("After push = {:?}", listener_path);
                let path = listener_path.as_path().to_str();
                match path {
                    Some(path) => {
                        println!("Path = {:?}", path);
                        Command::new(path)
                            .spawn()
                            .expect("sh command failed to start");
                        println!("Did it");
                        println!("Did it");
                        println!("Did it");
                        println!("Did it");
                        println!("Did it");
                        return Ok(());
                    }
                    None => return Err(Error::Registration(err)),
                }
                //Current dir: Ok("/home/rbsomeg101/github/arrows")
                //What = Command { cmd: "/home/rbsomeg101/github/arrows/target/debug/arrows", stdin: None, timeout: None }
                /***let cmd = Command::cargo_bin("arrows");
                match cmd {
                  Ok(what) => {
                      println!("What = {:?}", what);
                      println!("Here again: {:?}",err.kind());},
                  Err(err) => eprintln!("{}", err),
                }***/
            }
            _ => (),
        }
    }

    Ok(())
}
