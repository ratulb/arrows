use crate::constants::{INBOX, OUTBOX};
use crate::dbconnection::DBConnection;
use crate::signals::{DBEvent, EventBucket, Signal};
use rusqlite::hooks::Action;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

pub(crate) struct Publisher {
    publisher: Sender<Signal>,
    receiver: Option<Receiver<Signal>>,
    pub subscriber: Option<Subscriber>,
}
impl Publisher {
    pub fn new() -> Self {
        let (publisher, receiver) = channel();
        Self {
            publisher,
            receiver: Some(receiver),
            subscriber: None,
        }
    }
    pub fn start(&mut self, conn: &mut DBConnection) {
        conn.inner.update_hook(None::<fn(Action, &str, &str, i64)>);
        let publisher = self.publisher.clone();
        conn.inner
            .update_hook(Some(move |action: Action, _db: &str, tbl: &str, row_id| {
                let tbl_of_interest = tbl.starts_with(INBOX) || tbl.starts_with(OUTBOX);
                if action == Action::SQLITE_INSERT && tbl_of_interest {
                    let event = DBEvent(String::from(tbl), row_id);
                    publisher
                        .send(Signal::DbUpdate(event))
                        .expect("Event published");
                }
            }));
        let receiver = self.receiver.take();
        let mut subscriber = Subscriber::new(receiver);
        subscriber.start();
        self.subscriber = Some(subscriber);
    }

    pub fn loopbreak(&self) {
        self.publisher.send(Signal::Stop).expect("Sent");
    }
}
use crate::Mail;

pub fn transpose(rows: &mut Vec<Vec<Mail>>) -> Vec<Vec<Mail>> {
    assert!(!rows.is_empty());
    let size = rows
        .iter()
        .max_by(|row1, row2| row1.len().cmp(&row2.len()))
        .unwrap()
        .len();
    let rows = rows.iter_mut().map(|row| {
        row.resize_with(size, Mail::default);
        row
    });
    let mut result = vec![vec![Mail::Blank; rows.len()]; size];
    for (i, row) in rows.enumerate() {
        for (j, e) in row.drain(..).enumerate() {
            result[j][i] = e;
        }
    }
    result
}

pub(crate) struct Subscriber {
    receiver: Option<Receiver<Signal>>,
    pub join_handle: Option<JoinHandle<()>>,
}

impl Subscriber {
    pub fn new(receiver: Option<Receiver<Signal>>) -> Self {
        Self {
            receiver,
            join_handle: None,
        }
    }
    pub fn start(&mut self) {
        let receiver = self.receiver.take();
        let join_handle = std::thread::spawn(move || {
            let receiver = receiver.as_ref().expect("Inner receiver");
            let mut bucket = EventBucket::new();
            loop {
                let event = receiver.recv().expect("Expected event");
                match event {
                    Signal::Stop => break,
                    Signal::DbUpdate(evt) => {
                        println!("Received event = {:?}", evt);
                        bucket.add_event(evt);
                    }
                }
            }
        });
        self.join_handle = Some(join_handle);
    }
}
