use crate::{
    common::utils::{compute_hash, from_bytes, option_of_bytes},
    Addr,
};
use serde::{Deserialize, Serialize};
use std::mem::{replace, swap};
use std::time::SystemTime;
use uuid::Uuid;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Content {
    Text(String),
    Binary(Vec<u8>),
}

use Content::*;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Mail {
    Trade(Msg),
    Bulk(Vec<Msg>),
    Blank,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Default)]
pub struct Msg {
    id: u64,
    from: Option<Addr>,
    to: Option<Addr>,
    content: Option<Content>,
    dispatched: Option<SystemTime>,
}

impl Msg {
    pub fn new(content: Option<Vec<u8>>, from: &str, to: &str) -> Self {
        Self {
            id: compute_hash(&Uuid::new_v4()),
            from: Some(Addr::new(from)),
            to: Some(Addr::new(to)),
            content: content.map(|data| Binary(data)),
            dispatched: None,
        }
    }
    pub fn new_with_text(content: &str, from: &str, to: &str) -> Self {
        Self {
            id: compute_hash(&Uuid::new_v4()),
            from: Some(Addr::new(from)),
            to: Some(Addr::new(to)),
            content: Some(Text(content.to_string())),
            dispatched: None,
        }
    }

    pub fn content_as_text(&self) -> Option<&str> {
        match self.content {
            Some(Binary(ref bytes)) => {
                let text: crate::Result<&str> = from_bytes(bytes);
                text.ok()
            }
            Some(Text(ref s)) => Some(s),
            None => None,
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        option_of_bytes(self).unwrap_or_default()
    }

    pub fn uturn_with_text(&mut self, reply: &str) {
        swap(&mut self.from, &mut self.to);
        let _ignore = replace(&mut self.content, Some(Text(reply.to_string())));
    }

    pub fn update_text_content(&mut self, reply: &str) {
        let _ignore = replace(&mut self.content, Some(Text(reply.to_string())));
    }

    pub fn with_content_and_to(&mut self, new_content: Vec<u8>, new_to: &str) {
        self.content = Some(Binary(new_content));
        self.to = Some(Addr::new(new_to));
    }

    pub fn with_content(&mut self, new_content: Vec<u8>) {
        self.content = Some(Binary(new_content));
    }

    pub fn set_recipient(&mut self, new_to: &str) {
        self.to = Some(Addr::new(new_to));
    }
    pub fn set_recipient_ip(&mut self, new_to_ip: &str) {
        if let Some(ref mut addr) = self.to {
            addr.with_ip(new_to_ip);
        }
    }
    pub fn set_recipient_port(&mut self, new_port: u16) {
        if let Some(ref mut addr) = self.to {
            addr.with_port(new_port);
        }
    }

    pub fn uturn_with_reply(&mut self, reply: Option<Vec<u8>>) {
        swap(&mut self.from, &mut self.to);
        let _ignore = replace(&mut self.content, reply.map(|data| Binary(data)));
    }

    pub fn get_content(&self) -> Option<Vec<u8>> {
        match &self.content {
            Some(Binary(data)) => Some(data.to_vec()),
            _ => None,
        }
    }
    //Would take content out (if binary)- leaving message content to a None
    pub fn get_content_out(&mut self) -> Option<Vec<u8>> {
        match self.content.take() {
            Some(Binary(data)) => Some(data),
            content @ Some(_) => {
                self.content = content;
                None
            }
            _ => None,
        }
    }

    pub fn get_to(&self) -> &Option<Addr> {
        &self.to
    }
    pub fn get_id(&self) -> &u64 {
        &self.id
    }

    pub fn id_as_string(&self) -> String {
        self.get_id().to_string()
    }

    //Callers would be better served by cloning the returned &u64
    pub fn get_to_id(&self) -> u64 {
        match self.to {
            Some(ref addr) => addr.get_id(),
            None => 0,
        }
    }

    pub fn get_from(&self) -> &Option<Addr> {
        &self.from
    }
    pub fn is_outbound(&self) -> bool {
        match self.to {
            Some(ref addr) => !addr.is_local(),
            None => false,
        }
    }
}

impl Default for Mail {
    fn default() -> Self {
        Mail::Blank
    }
}

impl From<Msg> for Mail {
    fn from(msg: Msg) -> Self {
        Mail::Trade(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::utils::{from_bytes, option_of_bytes};

    #[test]
    fn create_trade_msg_test_content_and_to() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(
            from_bytes(&msg.get_content_out().unwrap()).ok(),
            Some("Content")
        );
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_to")));
    }

    #[test]
    fn create_trade_msg_test_from() {
        let msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_from(), &Some(Addr::new("addr_from")));
    }

    #[test]
    fn create_trade_msg_test_alter_content_and_to() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), option_of_bytes(&"Content"));
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_to")));
        msg.with_content_and_to(option_of_bytes(&"New content").unwrap(), "New_to");
        assert_eq!(msg.get_content(), option_of_bytes(&"New content"));
        assert_eq!(msg.get_to(), &Some(Addr::new("New_to")));
    }

    #[test]
    fn set_recipient_test_1() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_to")));
        msg.set_recipient("The new recipient");
        assert_eq!(msg.get_to(), &Some(Addr::new("The new recipient")));
    }

    #[test]
    fn uturn_with_reply_test_1() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        msg.uturn_with_reply(option_of_bytes(&"Reply"));
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_from")));
        assert_eq!(msg.get_from(), &Some(Addr::new("addr_to")));
        assert_eq!(msg.get_content(), option_of_bytes(&"Reply"));
    }
    #[test]
    fn uturn_with_text_reply_test_1() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), option_of_bytes(&"Content"));
        msg.uturn_with_text("Reply");
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_from")));
        assert_eq!(msg.get_from(), &Some(Addr::new("addr_to")));
        assert_eq!(msg.get_content(), option_of_bytes(&"Reply"));
    }
    #[test]
    fn content_as_text_test_1() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), option_of_bytes(&"Content"));
        assert_eq!(msg.content_as_text(), Some("Content"));

        msg.update_text_content("Updated content");
        assert_eq!(msg.get_content(), option_of_bytes(&"Updated content"));
        assert_eq!(msg.content_as_text(), Some("Updated content"));
    }
    #[test]
    fn outbound_mgs_test_1() {
        let mut trade_msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert!(!trade_msg.is_outbound());

        trade_msg.set_recipient_ip("89.89.89.89");
        assert!(trade_msg.is_outbound());
    }
}
