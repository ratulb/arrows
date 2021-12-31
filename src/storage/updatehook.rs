use crate::constants::{INBOX, OUTBOX};
use crate::dbconnection::DBConnection;
use crate::signals::DBEvent;
use rusqlite::hooks::Action;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

pub(crate) struct UpdateHook {
    sender: Sender<DBEvent>,
    receiver: Option<Receiver<DBEvent>>,
    join_handle: Option<JoinHandle<()>>,
}
impl UpdateHook {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver: Some(receiver),
            join_handle: None,
        }
    }
    pub fn attach(&mut self, conn: &mut DBConnection) -> Option<JoinHandle<()>> {
        conn.inner.update_hook(None::<fn(Action, &str, &str, i64)>);
        let sender = self.sender.clone();
        conn.inner
            .update_hook(Some(move |action: Action, _db: &str, tbl: &str, row_id| {
                let tbl_of_interest = tbl.starts_with(INBOX) || tbl.starts_with(OUTBOX);
                if action == Action::SQLITE_INSERT && tbl_of_interest {
                    let event = DBEvent(String::from(tbl), row_id);
                    sender.send(event).expect("Event published");
                }
            }));
        let receiver = self.receiver.take();
        let join_handle = std::thread::spawn(move || {
            let receiver = receiver.as_ref().expect("Inner receiver");
            loop {
                let event = receiver.recv().expect("Expected event");
                println!("Received event = {:?}", event);
            }
        });
        Some(join_handle)
    }
}
