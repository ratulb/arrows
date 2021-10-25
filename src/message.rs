use crate::{compute_hash, from_bytes, option_of_bytes, Address};
use serde::{Deserialize, Serialize};
use std::io::{Result, Seek, Write};
use std::mem::{replace, swap};
use std::time::SystemTime;
use uuid::Uuid;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    Custom {
        id: u64,
        from: Option<Address>,
        to: Option<Address>,
        content: Option<Vec<u8>>,
        recipients: Option<AdditionalRecipients>,
        created: SystemTime,
    },
    Internal {
        id: u64,
        from: Option<Address>,
        to: Option<Address>,
        content: Option<Vec<u8>>,
        recipients: Option<AdditionalRecipients>,
        created: SystemTime,
    },
    Blank,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum AdditionalRecipients {
    All,
    OnlySome(Vec<Address>),
}

impl  Message {
    pub fn new(content: Option<Vec<u8>>, from: &str, to: &str) -> Self {
        Self::Custom {
            id: compute_hash(&Uuid::new_v4()),
            from: Some(Address::new(from)),
            to: Some(Address::new(to)),
            content,
            recipients: None,
            created: SystemTime::now(),
        }
    }
    pub fn new_with_text(content: &str, from: &str, to: &str) -> Self {
        Self::Custom {
            id: compute_hash(&Uuid::new_v4()),
            from: Some(Address::new(from)),
            to: Some(Address::new(to)),
            content: option_of_bytes(content),
            recipients: None,
            created: SystemTime::now(),
        }
    }
    pub fn content_as_text(&self) -> Option<&str> {
        match self {
            Message::Custom { ref content, .. } => match content {
                Some(ref value) => {
                    let text: Result<&str> = from_bytes(value);
                    text.ok()
                }
                None => None,
            },
            Message::Internal { ref content, .. } => match content {
                Some(ref value) => {
                    let text: Result<&str> = from_bytes(value);
                    text.ok()
                }
                None => None,
            },
            Message::Blank => None,
        }
    }

    pub fn uturn_with_text(&mut self, reply: &str) {
        match self {
            Message::Custom {
                ref mut from,
                ref mut to,
                ref mut content,
                ..
            } => {
                swap(from, to);
                let _ignore = replace(content, option_of_bytes(reply));
            }
            Message::Internal {
                ref mut from,
                ref mut to,
                ref mut content,
                ..
            } => {
                swap(from, to);
                let _ignore = replace(content, option_of_bytes(reply));
            }
            Message::Blank => (),
        }
    }

    pub fn update_text_content(&mut self, reply: &str) {
        match self {
            Message::Custom {
                ref mut content, ..
            } => {
                let _ignore = replace(content, option_of_bytes(reply));
            }
            Message::Internal {
                ref mut content, ..
            } => {
                let _ignore = replace(content, option_of_bytes(reply));
            }
            Message::Blank => (),
        }
    }

    pub fn with_content_and_to(&mut self, new_content: Vec<u8>, new_to: &str) {
        match self {
            Message::Custom {
                ref mut content,
                ref mut to,
                ..
            } => {
                *content = Some(new_content);
                *to = Some(Address::new(new_to));
            }
            Message::Internal {
                ref mut content,
                ref mut to,
                ..
            } => {
                *content = Some(new_content);
                *to = Some(Address::new(new_to));
            }
            Message::Blank => (),
        }
    }

    pub fn with_content(&mut self, new_content: Vec<u8>) {
        match self {
            Message::Custom {
                ref mut content, ..
            } => {
                *content = Some(new_content);
            }
            Message::Internal {
                ref mut content, ..
            } => {
                *content = Some(new_content);
            }
            Message::Blank => (),
        }
    }

    pub fn set_recipient(&mut self, new_to: &str) {
        match self {
            Message::Custom { ref mut to, .. } => {
                *to = Some(Address::new(new_to));
            }
            Message::Internal { ref mut to, .. } => {
                *to = Some(Address::new(new_to));
            }
            Message::Blank => (),
        }
    }
    pub fn set_recipient_ip(&mut self, new_to_ip: &str) {
        match self {
            Message::Custom { to, .. } => match to {
                Some(ref mut addr) => addr.with_ip(new_to_ip),
                None => (),
            },
            Message::Internal { to, .. } => match to {
                Some(ref mut addr) => addr.with_ip(new_to_ip),
                None => (),
            },
            Message::Blank => (),
        }
    }
    pub fn set_recipient_port(&mut self, new_port: u16) {
        match self {
            Message::Custom { to, .. } => match to {
                Some(ref mut addr) => addr.with_port(new_port),
                None => (),
            },
            Message::Internal { to, .. } => match to {
                Some(ref mut addr) => addr.with_port(new_port),
                None => (),
            },
            Message::Blank => (),
        }
    }

    pub fn uturn_with_reply(&mut self, reply: Option<Vec<u8>>) {
        match self {
            Message::Custom {
                ref mut from,
                ref mut to,
                ref mut content,
                ..
            } => {
                swap(from, to);
                let _ignore = replace(content, reply);
            }
            Message::Internal {
                ref mut from,
                ref mut to,
                ref mut content,
                ..
            } => {
                swap(from, to);
                let _ignore = replace(content, reply);
            }
            Message::Blank => (),
        }
    }

    pub fn with_recipients(&mut self, new_recipients: Vec<&str>) {
        match self {
            Message::Custom {
                ref mut recipients, ..
            } => {
                *recipients = Some(AdditionalRecipients::OnlySome(
                    new_recipients
                        .iter()
                        .map(|each| Address::new(each))
                        .collect(),
                ));
            }
            Message::Internal {
                ref mut recipients, ..
            } => {
                *recipients = Some(AdditionalRecipients::OnlySome(
                    new_recipients
                        .iter()
                        .map(|each| Address::new(each))
                        .collect(),
                ));
            }
            Message::Blank => (),
        }
    }

    pub fn set_recipient_all(&mut self) {
        match self {
            Message::Custom {
                ref mut recipients, ..
            } => {
                *recipients = Some(AdditionalRecipients::All);
            }
            Message::Internal {
                ref mut recipients, ..
            } => {
                *recipients = Some(AdditionalRecipients::All);
            }
            Message::Blank => (),
        }
    }

    pub fn get_content(&self) -> &Option<Vec<u8>> {
        match self {
            Message::Custom { content, .. } => content,
            Message::Internal { content, .. } => content,
            Message::Blank => &None,
        }
    }
    //Would take content out - leaving message content to a None
    pub fn get_content_out(&mut self) -> Option<Vec<u8>> {
        match self {
            Message::Custom {
                ref mut content, ..
            } => content.take(),
            Message::Internal {
                ref mut content, ..
            } => content.take(),
            Message::Blank => None,
        }
    }

    pub fn get_to(&self) -> &Option<Address> {
        match self {
            Message::Custom { to, .. } => to,
            Message::Internal { to, .. } => to,
            Message::Blank => &None,
        }
    }
    pub fn get_id(&self) -> &u64 {
        match self {
            Message::Custom { id, .. } => id,
            Message::Internal { id, .. } => id,
            Message::Blank => &0,
        }
    }
    //Callers would be better served by cloning the returned &u64
    pub fn get_to_id(&self) -> u64 {
        match self {
            Message::Custom { to, .. } => match to {
                Some(ref addr) => addr.get_id(),
                None => 0,
            },
            Message::Internal { to, .. } => match to {
                Some(ref addr) => addr.get_id(),
                None => 0,
            },
            Message::Blank => 0,
        }
    }

    pub fn get_from(&self) -> &Option<Address> {
        match self {
            Message::Custom { from, .. } => from,
            Message::Internal { from, .. } => from,
            Message::Blank => &None,
        }
    }
    pub fn is_outbound(&self) -> bool {
        match self {
            Message::Custom { to, .. } => match to {
                Some(ref addr) => !addr.is_local(),
                None => false,
            },
            Message::Internal { to, .. } => match to {
                Some(ref addr) => !addr.is_local(),
                None => false,
            },
            Message::Blank => false,
        }
    }

    pub fn get_recipients(&self) -> &Option<AdditionalRecipients> {
        match self {
            Message::Custom { recipients, .. } => recipients,
            Message::Internal { recipients, .. } => recipients,
            Message::Blank => &None,
        }
    }

    pub fn is_recipient_all(&self) -> bool {
        match self {
            Message::Custom { ref recipients, .. } => {
                matches!(*recipients, Some(AdditionalRecipients::All))
            }
            Message::Internal { ref recipients, .. } => {
                matches!(*recipients, Some(AdditionalRecipients::All))
            }
            Message::Blank => false,
        }
    }

    pub(crate) fn internal(content: Option<Vec<u8>>, from: &str, to: &str) -> Self {
        Self::Internal {
            id: compute_hash(&Uuid::new_v4()),
            from: Some(Address::new(from)),
            to: Some(Address::new(to)),
            content,
            recipients: None,
            created: SystemTime::now(),
        }
    }
}

impl Default for Message {
    fn default() -> Self {
        Message::Blank
    }
}

impl  Message {
    pub async fn write<W: Seek + Write>(&self, w: &mut W) -> Result<()> {
        serde_json::to_writer(w, self)?;
        Ok(())
    }
}
impl  Message {
    pub fn write_sync<W: Seek + Write>(&self, w: &mut W) -> Result<()> {
        serde_json::to_writer(w, self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::from_bytes;
    use crate::option_of_bytes;
    use crate::type_of;
    use std::fs::OpenOptions;
    use std::io::BufWriter;
    #[test]
    fn create_custom_msg_test_content_and_to() {
        let mut msg = Message::new(option_of_bytes("Content"), "addr_from", "addr_to");
        assert_eq!(
            from_bytes(&msg.get_content_out().unwrap()).ok(),
            Some("Content")
        );
        assert_eq!(msg.get_to(), &Some(Address::new("addr_to")));
    }

    #[test]
    fn create_internal_msg_test_content_and_to() {
        let msg = Message::internal(option_of_bytes("Content"), "addr_from", "addr_to");
        assert_eq!(*msg.get_content(), option_of_bytes("Content"));
        assert_eq!(msg.get_to(), &Some(Address::new("addr_to")));
    }

    #[test]
    fn create_custom_msg_test_from() {
        let msg = Message::new(option_of_bytes("Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_from(), &Some(Address::new("addr_from")));
    }

    #[test]
    fn create_internal_msg_test_from() {
        let msg = Message::internal(option_of_bytes("Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_from(), &Some(Address::new("addr_from")));
    }

    #[test]
    fn create_custom_msg_test_alter_content_and_to() {
        let mut msg = Message::new(option_of_bytes("Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &option_of_bytes("Content"));
        assert_eq!(msg.get_to(), &Some(Address::new("addr_to")));
        msg.with_content_and_to(option_of_bytes("New content").unwrap(), "New_to");
        assert_eq!(msg.get_content(), &option_of_bytes("New content"));
        assert_eq!(msg.get_to(), &Some(Address::new("New_to")));
    }

    #[test]
    fn create_internal_msg_test_alter_content_and_to() {
        let mut msg = Message::internal(option_of_bytes("Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &option_of_bytes("Content"));
        assert_eq!(msg.get_to(), &Some(Address::new("addr_to")));
        msg.with_content_and_to(option_of_bytes("New content").unwrap(), "New_to");
        assert_eq!(msg.get_content(), &option_of_bytes("New content"));
        assert_eq!(msg.get_to(), &Some(Address::new("New_to")));
    }

    #[test]
    fn alter_additional_recipients_test_1() {
        let mut msg = Message::internal(option_of_bytes("Content"), "addr_from", "addr_to");
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
        let mut msg = Message::internal(option_of_bytes("Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_recipients(), &None);
        msg.set_recipient_all();
        type_of(&msg.get_recipients());
        assert_eq!(msg.get_recipients(), &Some(AdditionalRecipients::All));
    }

    #[test]
    fn set_recipient_test_1() {
        let mut msg = Message::internal(option_of_bytes("Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_to(), &Some(Address::new("addr_to")));
        msg.set_recipient("The new recipient");
        assert_eq!(msg.get_to(), &Some(Address::new("The new recipient")));
    }

    #[test]
    fn set_complex_msg_test_1() {
        #[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
        struct Complex<T> {
            inner: T,
            elems: Vec<Simple>,
        }
        #[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
        struct Inner {
            name: String,
            children: Vec<String>,
            male: bool,
            age: u8,
        }
        #[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
        struct Simple {
            e1: i32,
            e2: usize,
            e3: Option<bool>,
        }
        let simple = Simple {
            e1: 42,
            e2: 999,
            e3: Some(false),
        };

        let inner = Inner {
            name: "Some body".to_string(),
            children: vec!["Some value".to_string()],
            male: true,
            age: 99,
        };

        let complex = Complex {
            inner,
            elems: vec![simple],
        };

        let msg = Message::internal(option_of_bytes(&complex), "addr_from", "addr_to");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("msg.txt")
            .expect("Complex msg write failure");
        let mut bufwriter = BufWriter::new(file);
        assert_eq!(msg.write_sync(&mut bufwriter).expect("Should get ()"), ());
    }

    #[test]
    fn uturn_with_reply_test_1() {
        let mut msg = Message::internal(option_of_bytes("Content"), "addr_from", "addr_to");
        msg.uturn_with_reply(option_of_bytes("Reply"));
        assert_eq!(msg.get_to(), &Some(Address::new("addr_from")));
        assert_eq!(msg.get_from(), &Some(Address::new("addr_to")));
        assert_eq!(msg.get_content(), &option_of_bytes("Reply"));
    }
    #[test]
    fn uturn_with_text_reply_test_1() {
        let mut msg = Message::internal(option_of_bytes("Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &option_of_bytes("Content"));
        msg.uturn_with_text("Reply");
        assert_eq!(msg.get_to(), &Some(Address::new("addr_from")));
        assert_eq!(msg.get_from(), &Some(Address::new("addr_to")));
        assert_eq!(msg.get_content(), &option_of_bytes("Reply"));
    }
    #[test]
    fn content_as_text_test_1() {
        let mut msg = Message::internal(option_of_bytes("Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &option_of_bytes("Content"));
        assert_eq!(msg.content_as_text(), Some("Content"));

        msg.update_text_content("Updated content");
        assert_eq!(msg.get_content(), &option_of_bytes("Updated content"));
        assert_eq!(msg.content_as_text(), Some("Updated content"));

        let mut msg = Message::new(option_of_bytes("Custom content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &option_of_bytes("Custom content"));
        assert_eq!(msg.content_as_text(), Some("Custom content"));

        msg.update_text_content("New updated content");
        assert_eq!(msg.get_content(), &option_of_bytes("New updated content"));
        assert_eq!(msg.content_as_text(), Some("New updated content"));
    }
    #[test]
    fn outbound_mgs_test_1() {
        let internal_msg = Message::internal(option_of_bytes("Content"), "addr_from", "addr_to");
        assert_eq!(internal_msg.is_outbound(), false);
        let custom_msg = Message::new(option_of_bytes("Content"), "addr_from", "addr_to");
        assert_eq!(custom_msg.is_outbound(), false);
        let blank = Message::Blank;
        assert_eq!(blank.is_outbound(), false);

        let mut internal_msg =
            Message::internal(option_of_bytes("Content"), "addr_from", "addr_to");
        internal_msg.set_recipient_ip("89.89.89.89");

        assert_eq!(internal_msg.is_outbound(), true);
        let mut custom_msg = Message::new(option_of_bytes("Content"), "addr_from", "addr_to");

        custom_msg.set_recipient_ip("89.89.89.89");
        assert_eq!(custom_msg.is_outbound(), true);
        let blank = Message::Blank;
        assert_eq!(blank.is_outbound(), false);
    }
}
