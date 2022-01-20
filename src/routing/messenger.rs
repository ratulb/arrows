//!`Messenger` sends out  messages to remote or local system. Tries to boot up
//!listener binary in case of local connection failre.
//!
//!Uses a tcp client which serializes collection of messages. Each collection of messages
//!ends with byte marks <https://github.com/ratulb/byte_marks>. This is how the receiving
//!end reconstruct messages back. Message serialization and deserialization is based on
//![bincode] <https://github.com/bincode-org/bincode> library.

use crate::common::config::Config;
use crate::routing::messenger::client::Client;
use crate::{Action, Addr, Error::MsgSendError, Mail, Msg, Result};
use std::collections::HashMap;
use std::io::ErrorKind::ConnectionRefused;
use std::net::SocketAddr;
use std::process::Command;
use std::thread;
use std::time::Duration;

///The client face of actor system. Sends out messages with text or binary(v8) as payload.
pub(crate) struct Messenger;

impl Messenger {
    pub(crate) fn send(mut mails: HashMap<&Addr, Vec<Msg>>) -> Result<()> {
        mails.iter_mut().for_each(|(addr, msgs)| {
            msgs.iter_mut().for_each(|msg| {
                msg.set_recipient_addr(addr);
            });
            if let Some(host_addr) = addr.get_socket_addr() {
                match Client::connect(host_addr) {
                    Ok(mut client) => {
                        if let Err(err) = client.send(msgs) {
                            println!("{}", err);
                        } else {
                            println!("Messages sent to host {}", host_addr);
                        }
                    }
                    Err(err) => {
                        eprintln!("Host: {} {}", host_addr, err);
                        let _rs = Self::handle_err(msgs, host_addr, err);
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
                let _rs = match Client::connect(host_addr) {
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

    fn group_by(msgs: Vec<Msg>) -> HashMap<SocketAddr, Vec<Msg>> {
        let mut groups: HashMap<SocketAddr, Vec<Msg>> = HashMap::new();
        for msg in msgs {
            let host_addr = match msg.get_to() {
                Some(ref addr) => addr.get_socket_addr().expect("host"),
                None => continue,
            };
            groups.entry(host_addr).or_default().push(msg);
        }
        groups
    }

    fn handle_err(msgs: &mut Vec<Msg>, addr: SocketAddr, err: std::io::Error) -> Result<()> {
        match err.kind() {
            ConnectionRefused => Self::try_bootup_and_resend(msgs, addr),
            _ => {
                eprintln!("{}", err);
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
                if let Err(err) = Self::bootup() {
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
                    println!("{}", String::from_utf8_lossy(&buf[..len]));
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
