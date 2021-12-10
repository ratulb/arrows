use crate::common::addr::Addr;
use crate::common::utils::{compute_hash, from_bytes, option_of_bytes};
use serde::{Deserialize, Serialize};
use std::mem::{replace, swap};
use std::time::SystemTime;
use uuid::Uuid;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Mail {
    Trade(Msg),
    Blank,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum AdditionalRecipients {
    All,
    OnlySome(Vec<Addr>),
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Default)]
pub struct Msg {
    id: u64,
    from: Option<Addr>,
    to: Option<Addr>,
    content: Option<Vec<u8>>,
    recipients: Option<AdditionalRecipients>,
    dispatched: Option<SystemTime>,
}

impl Msg {
    pub fn new(content: Option<Vec<u8>>, from: &str, to: &str) -> Self {
        Self {
            id: compute_hash(&Uuid::new_v4()),
            from: Some(Addr::new(from)),
            to: Some(Addr::new(to)),
            content,
            recipients: None,
            dispatched: None,
        }
    }
    pub fn new_with_text(content: &str, from: &str, to: &str) -> Self {
        Self {
            id: compute_hash(&Uuid::new_v4()),
            from: Some(Addr::new(from)),
            to: Some(Addr::new(to)),
            content: option_of_bytes(&String::from(content)),
            recipients: None,
            dispatched: None,
        }
    }

    pub fn content_as_text(&self) -> Option<&str> {
        match self.content {
            Some(ref value) => {
                let text: crate::Result<&str> = from_bytes(value);
                text.ok()
            }
            None => None,
        }
    }
    pub fn get_bytes(&self) -> Vec<u8> {
        option_of_bytes(self).unwrap_or_default()
    }

    pub fn uturn_with_text(&mut self, reply: &str) {
        swap(&mut self.from, &mut self.to);
        let _ignore = replace(&mut self.content, option_of_bytes(&String::from(reply)));
    }

    pub fn update_text_content(&mut self, reply: &str) {
        let _ignore = replace(&mut self.content, option_of_bytes(&String::from(reply)));
    }

    pub fn with_content_and_to(&mut self, new_content: Vec<u8>, new_to: &str) {
        self.content = Some(new_content);
        self.to = Some(Addr::new(new_to));
    }

    pub fn with_content(&mut self, new_content: Vec<u8>) {
        self.content = Some(new_content);
    }

    pub fn set_recipient(&mut self, new_to: &str) {
        self.to = Some(Addr::new(new_to));
    }
    pub fn set_recipient_ip(&mut self, new_to_ip: &str) {
        match self.to {
            Some(ref mut addr) => addr.with_ip(new_to_ip),
            None => (),
        }
    }
    pub fn set_recipient_port(&mut self, new_port: u16) {
        match self.to {
            Some(ref mut addr) => addr.with_port(new_port),
            None => (),
        }
    }

    pub fn uturn_with_reply(&mut self, reply: Option<Vec<u8>>) {
        swap(&mut self.from, &mut self.to);
        let _ignore = replace(&mut self.content, reply);
    }

    pub fn with_recipients(&mut self, new_recipients: Vec<&str>) {
        self.recipients = Some(AdditionalRecipients::OnlySome(
            new_recipients.iter().map(|each| Addr::new(each)).collect(),
        ));
    }

    pub fn set_recipient_all(&mut self) {
        self.recipients = Some(AdditionalRecipients::All);
    }

    pub fn get_content(&self) -> &Option<Vec<u8>> {
        &self.content
    }
    //Would take content out - leaving message content to a None
    pub fn get_content_out(&mut self) -> Option<Vec<u8>> {
        self.content.take()
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

    pub fn get_recipients(&self) -> &Option<AdditionalRecipients> {
        &self.recipients
    }

    pub fn is_recipient_all(&self) -> bool {
        matches!(self.recipients, Some(AdditionalRecipients::All))
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
    use crate::from_bytes;
    use crate::option_of_bytes;
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
        assert_eq!(msg.get_content(), &option_of_bytes(&"Content"));
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_to")));
        msg.with_content_and_to(option_of_bytes(&"New content").unwrap(), "New_to");
        assert_eq!(msg.get_content(), &option_of_bytes(&"New content"));
        assert_eq!(msg.get_to(), &Some(Addr::new("New_to")));
    }

    #[test]
    fn alter_additional_recipients_test_1() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        let additional_recipients = vec!["Recipient1", "Recipient2", "Recipient3"];
        msg.with_recipients(additional_recipients);
        let additional_recipients_returned = vec!["Recipient1", "Recipient2", "Recipient3"];
        if let Some(AdditionalRecipients::OnlySome(recipients)) = msg.get_recipients() {
            let mut index = 0;
            recipients.iter().for_each(|addr| {
                assert_eq!(addr.get_name(), additional_recipients_returned[index]);
                index += 1;
            });
        } else {
            panic!("Failed for message test - additional recipients");
        }
    }

    #[test]
    fn set_recipients_all_test_1() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_recipients(), &None);
        msg.set_recipient_all();
        assert_eq!(msg.get_recipients(), &Some(AdditionalRecipients::All));
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
        assert_eq!(msg.get_content(), &option_of_bytes(&"Reply"));
    }
    #[test]
    fn uturn_with_text_reply_test_1() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &option_of_bytes(&"Content"));
        msg.uturn_with_text("Reply");
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_from")));
        assert_eq!(msg.get_from(), &Some(Addr::new("addr_to")));
        assert_eq!(msg.get_content(), &option_of_bytes(&"Reply"));
    }
    #[test]
    fn content_as_text_test_1() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &option_of_bytes(&"Content"));
        assert_eq!(msg.content_as_text(), Some("Content"));

        msg.update_text_content("Updated content");
        assert_eq!(msg.get_content(), &option_of_bytes(&"Updated content"));
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
