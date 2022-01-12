use crate::catalog::ingress;
use crate::{Addr, Mail, Msg, Result};
use std::collections::HashMap;

pub(crate) struct Messenger;
impl Messenger {
    pub(crate) fn send(messages: HashMap<&Addr, Vec<Msg>>) -> Result<()> {
        let mut ins = 0;
        let mut outs = 0;
        for addr in messages.keys() {
            let len = messages.get(addr).unwrap_or(&Vec::new()).len();
            if addr.is_local() {
                ins += len;
            } else {
                outs += len;
            }
        }
        let mut ins = Vec::with_capacity(ins);
        let mut outs = Vec::with_capacity(outs);

        for (addr, mut msgs) in messages.into_iter() {
            for msg in msgs.iter_mut() {
                msg.set_recipient_add(addr);
            }
            if addr.is_local() {
                ins.extend(msgs);
            } else {
                outs.extend(msgs);
            }
        }
        if !ins.is_empty() {
            ingress(Mail::Bulk(ins));
        }
        if !outs.is_empty() {
            let _groups = Self::group_by(outs);
        }
        println!("I am very much alive and kicking!");
        Ok(())
    }
    pub(crate) fn mail(mail: Mail) -> Result<()> {
        let split = Mail::split(mail);
        match split {
            Some((ins, outs)) => {
                if !ins.is_empty() {
                    ingress(Mail::Bulk(ins));
                }
                if !outs.is_empty() {
                    let _groups = Self::group_by(outs);
                }
            }
            _ => eprintln!("Invalid input"),
        }
        Ok(())
    }
    fn group_by(outs: Vec<Msg>) -> HashMap<Addr, Vec<Msg>> {
        let mut groups: HashMap<Addr, Vec<Msg>> = HashMap::new();
        for msg in outs {
            groups
                .entry(msg.get_to().as_ref().unwrap().clone()) //Can this clone be avoided
                .or_default()
                .push(msg);
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
}
