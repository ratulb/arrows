use crate::catalog::send_off;
use crate::catalog::{self};
use crate::{Addr, DetailedMsg, Mail, Msg, Result};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
pub(crate) struct Delegate {
    receiver: Option<Arc<Mutex<Receiver<DetailedMsg>>>>,
}

impl Delegate {
    pub(crate) fn new(receiver: Arc<Mutex<Receiver<DetailedMsg>>>) -> Self {
        Self {
            receiver: Some(receiver),
        }
    }
    pub(crate) fn start(&mut self) -> JoinHandle<()> {
        let receiver = self.receiver.take().expect("Receiver");

        thread::spawn(move || loop {
            let receiver = receiver.lock();
            match receiver.recv() {
                Ok(msg) => {
                    println!(
                        "Received a message = {:?} {:?}",
                        std::thread::current().id(),
                        msg.0.get_to()
                    );
                    catalog::handle_invocation(msg);
                }
                Err(err) => {
                    eprintln!("Error receiving msg {:?}", err);
                    continue;
                }
            }
        })
    }
}

pub(crate) struct Router {
    sender: Sender<DetailedMsg>,
    delegates: Vec<JoinHandle<()>>,
}

impl Router {
    pub(crate) fn new(count: usize) -> Self {
        assert!(count > 0);
        let (sender, receiver) = channel();
        let mut delegates = Vec::with_capacity(count);
        let receiver = Arc::new(Mutex::new(receiver));
        for i in 0..count {
            println!("Delegate started = {:?}", i);
            delegates.push(Delegate::new(Arc::clone(&receiver)).start());
        }
        Self { sender, delegates }
    }

    pub(crate) fn route(&mut self, msgs: Vec<DetailedMsg>) {
        for msg in msgs {
            self.sender.send(msg).expect("Routing messages");
        }
    }
}
impl Drop for Router {
    fn drop(&mut self) {
        /*** for handle in std::mem::take(&mut self.delegates) {
            handle.join();
        }***/
    }
}

pub(crate) struct Messenger;
impl Messenger {
    pub(crate) fn send(messages: HashMap<&Addr, Vec<Msg>>) -> Result<()> {
        for (addr, mut msgs) in messages.into_iter() {
            for msg in msgs.iter_mut() {
                msg.set_recipient_add(addr);
            }
            if addr.is_local() {
                send_off(Mail::Bulk(msgs));
                println!("I am very much alive and kicking!");
            } else {
                //TODO send_off_remote//In fact everything should hit the server
            }
        }
        Ok(())
    }
}
