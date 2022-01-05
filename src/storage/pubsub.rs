use crate::constants::{INBOX, OUTBOX};
use crate::dbconnection::DBConnection;
use crate::events::{DBEvent, Events};
use crate::routers::Router;
use rusqlite::hooks::Action;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

pub(crate) struct Publisher {
    publisher: Sender<Events>,
    receiver: Option<Receiver<Events>>,
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
                        .send(Events::DbUpdate(event))
                        .expect("Event published");
                }
            }));
        let receiver = self.receiver.take();
        let mut subscriber = Subscriber::new(receiver);
        subscriber.start();
        self.subscriber = Some(subscriber);
    }

    pub fn loopbreak(&self) {
        self.publisher.send(Events::Stop).expect("Sent");
    }
}

pub(crate) struct Subscriber {
    receiver: Option<Receiver<Events>>,
    pub join_handle: Option<JoinHandle<()>>,
}

impl Subscriber {
    pub fn new(receiver: Option<Receiver<Events>>) -> Self {
        Self {
            receiver,
            join_handle: None,
        }
    }
    pub fn start(&mut self) {
        let receiver = self.receiver.take();
        let join_handle = std::thread::spawn(move || {
            let receiver = receiver.as_ref().expect("Inner receiver");
            let mut router = Router::new();
            loop {
                let event = receiver.recv().expect("Expected event");
                match event {
                    Events::Stop => break,
                    Events::DbUpdate(evt) => {
                        router.route(evt);
                    }
                }
            }
        });
        self.join_handle = Some(join_handle);
    }
}
