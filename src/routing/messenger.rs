use crate::common::config::Config;
use crate::routing::messenger::client::Client;
use crate::{Action, Addr, Error::MsgSendError, Mail, Msg, Result};
use std::collections::HashMap;
use std::io::ErrorKind::ConnectionRefused;
use std::net::SocketAddr;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub(crate) struct Messenger;

impl Messenger {
    pub(crate) fn send(mut mails: HashMap<&Addr, Vec<Msg>>) -> Result<()> {
        mails.iter_mut().for_each(|(addr, msgs)| {
            println!("The messages {:?}", msgs);
            println!("The messages {:?}", msgs);
            println!("The messages {:?}", msgs);
            println!("The messages {:?}", msgs);
            println!("The messages {:?}", msgs);
            println!("The messages {:?}", msgs);
            println!("The messages {:?}", msgs);
            msgs.iter_mut().for_each(|msg| {
                msg.set_recipient_addr(addr);
            });
            if let Some(host_addr) = addr.get_socket_addr() {
                match Client::connect(host_addr) {
                    Ok(mut client) => {
                        client.send(msgs);
                        println!("Messenger sent to {}", host_addr);
                    }
                    Err(err) => {
                        eprintln!("Host: {} {}", host_addr, err);
                        Self::handle_err(msgs, host_addr, err);
                    }
                }
            }
        });
        Ok(())
    }

    pub(crate) fn mail(mail: Mail) -> Result<()> {
        Self::group_by(mail.take_all())
            .into_iter()
            .for_each(|(host_addr, mut msgs)| {
                match Client::connect(host_addr) {
                    Ok(mut client) => match client.send(&mut msgs) {
                        Ok(ok) => {
                            println!("Messages sent to {}", host_addr);
                            Ok(ok)
                        }
                        Err(err) => {
                            eprintln!("{}", err);
                            Err(err.into())
                        }
                    },
                    Err(err) => {
                        eprintln!("Host: {} {}", host_addr, err);
                        Self::handle_err(&mut msgs, host_addr, err)
                    }
                };
            });
        Ok(())
    }

    fn handle_err(msgs: &mut Vec<Msg>, addr: SocketAddr, err: std::io::Error) -> Result<()> {
        match err.kind() {
            ConnectionRefused => Self::try_bootup_and_resend(msgs, addr),
            _ => {
                eprintln!("Unhandled error {}", err);
                Err(MsgSendError(err))
            }
        }
    }
    fn try_bootup_and_resend(msgs: &mut Vec<Msg>, socket_addr: SocketAddr) -> Result<()> {
        if Addr::is_ip_local(socket_addr.ip()) {
            if !msgs.is_empty()
                && (msgs[0].command_equals(Action::Shutdown)
                    || msgs[0].command_equals(Action::Echo("".to_string())))
            {
                return Ok(());
            } else {
                if let Err(err) = bootup() {
                    eprintln!("Bootup error {:?}", err);
                }
                let mail: Mail = std::mem::take(msgs).into();
                thread::sleep(Duration::from_millis(100));
                if let Err(err) = Messenger::mail(mail) {
                    eprintln!("Messenger mail error {:?}", err);
                }
            }
        }
        Ok(())
    }

    fn group_by(msgs: Vec<Msg>) -> HashMap<SocketAddr, Vec<Msg>> {
        let mut groups: HashMap<SocketAddr, Vec<Msg>> = HashMap::new();
        for msg in msgs {
            let host_addr = match msg.get_to() {
                Some(ref addr) => addr.get_socket_addr().expect("host"),
                None => {
                    println!("Filtering out none");
                    continue;
                }
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

        pub fn send(&mut self, msgs: &mut Vec<Msg>) -> Result<()> {
            let bulk = Mail::Bulk(std::mem::take(msgs));
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

///Boots up the msg listener(`MessageListener`) binary. The resident binary path is
///configurable via the environment variable 'resident_listener'.

pub fn bootup() -> Result<()> {
    let mut resident_listener = std::env::current_dir()?;
    resident_listener.push(Config::get_shared().resident_listener());
    let path = resident_listener.as_path().to_str();
    match path {
        Some(path) => {
            Command::new(path).spawn()?;
        }
        None => eprintln!("Listener binary could not be found in {:?}", path),
    }
    Ok(())
}
