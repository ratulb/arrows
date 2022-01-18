pub mod listener;
pub mod messenger;
use crate::catalog::{self};
use crate::RichMail;
use parking_lot::Mutex;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

pub(crate) struct Delegate {
    receiver: Option<Arc<Mutex<Receiver<RichMail>>>>,
}

impl Delegate {
    pub(crate) fn new(receiver: Arc<Mutex<Receiver<RichMail>>>) -> Self {
        Self {
            receiver: Some(receiver),
        }
    }
    pub(crate) fn start(&mut self) -> JoinHandle<()> {
        let receiver = self.receiver.take().expect("Receiver");
        thread::spawn(move || loop {
            let receiver = receiver.lock();
            match receiver.recv() {
                Ok(rich_mail) => {
                    catalog::handle_invocation(rich_mail);
                }
                Err(err) => {
                    eprintln!("Error receiving msg {}", err);
                    continue;
                }
            }
        })
    }
}

pub(crate) struct Router {
    sender: Sender<RichMail>,
    delegates: Vec<JoinHandle<()>>,
}

impl Router {
    pub(crate) fn new(count: usize) -> Self {
        assert!(count > 0);
        let (sender, receiver) = channel();
        let mut delegates = Vec::with_capacity(count);
        let receiver = Arc::new(Mutex::new(receiver));
        for i in 0..count {
            println!("Delegate started = {}", i);
            delegates.push(Delegate::new(Arc::clone(&receiver)).start());
        }
        Self { sender, delegates }
    }

    pub(crate) fn route(&mut self, msgs: Vec<RichMail>) {
        for msg in msgs {
            match self.sender.send(msg) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("Router: error routing message {}", err);
                }
            }
        }
    }
}
impl Drop for Router {
    fn drop(&mut self) {
        for handle in std::mem::take(&mut self.delegates) {
            handle.join();
        }
    }
}
