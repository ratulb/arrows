

use crate::DetailedMsg;
use parking_lot::Mutex;
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
            let receiver = receiver.lock(); /*** {
                                                Ok(receiver) => receiver,
                                                Err(poisoned) => poisoned.into_inner(),
                                            };***/
            match receiver.recv() {
                Ok(_msg) => {
                    /***println!("Is it locked here = {:?}", CTX.is_locked());
                    let mutex = CTX.lock();
                    println!("Is it locked here ???= {:?}", CTX.is_locked());
                    println!("Here are the actors = {:?}", mutex.borrow().actors);
                    let actor: Option<Box<dyn Actor>> = None; //mutex.borrow().actors.get_actor(*msg.0.get_id());
                    match actor {
                        Some(_actor) => println!("Found actor"),
                        None => {
                            parking_lot::MutexGuard::unlock_fair(mutex);
                            eprintln!("Actor not found - reloading..");
                            crate::catalog::reload_actor(*msg.0.get_id());
                            eprintln!("Done - reloading..");
                        }
                    }***/
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
            //delegates.push(Delegate::new(Arc::clone(&receiver)).start());
            delegates.push(Delegate::new(receiver.clone()).start());
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
