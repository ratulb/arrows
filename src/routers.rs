use crate::signals::Signal;
use std::sync::mpsc::Receiver;

pub(crate) struct InboxRouter;
pub(crate) struct OutboxRouter;
pub(crate) struct ExternalRouter;

pub(crate) struct Router {
    receiver: Receiver<Signal>,
}
