use crate::{option_of_bytes, Address};
use serde::{Deserialize, Serialize};
use std::io::{Result, Seek, Write};
use std::mem::{replace, swap};
use std::time::SystemTime;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Message<'a> {
    Custom {
        #[serde(borrow)]
        from: Option<Address<'a>>,
        #[serde(borrow)]
        to: Option<Address<'a>>,
        content: Option<Vec<u8>>,
        #[serde(borrow)]
        recipients: Option<AdditionalRecipients<'a>>,
        created: SystemTime,
    },
    Internal {
        #[serde(borrow)]
        from: Option<Address<'a>>,
        #[serde(borrow)]
        to: Option<Address<'a>>,
        content: Option<Vec<u8>>,
        #[serde(borrow)]
        recipients: Option<AdditionalRecipients<'a>>,
        created: SystemTime,
    },
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum AdditionalRecipients<'a> {
    All,
    #[serde(borrow)]
    OnlySome(Vec<Address<'a>>),
}

impl<'a> Message<'a> {
    pub fn new(content: Option<Vec<u8>>, from: &'a str, to: &'a str) -> Self {
        Self::Custom {
            from: Some(Address::new(from)),
            to: Some(Address::new(to)),
            content,
            recipients: None,
            created: SystemTime::now(),
        }
    }
    pub fn new_with_text(content: &str, from: &'a str, to: &'a str) -> Self {
        Self::Custom {
            from: Some(Address::new(from)),
            to: Some(Address::new(to)),
            content: option_of_bytes(content),
            recipients: None,
            created: SystemTime::now(),
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
        }
    }

    pub fn with_content_and_to(
        //&'a mut self,
        &mut self,
        new_content: Vec<u8>,
        new_to: &'a str,
    ) {
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
        }
    }

    pub fn with_content(&mut self, new_content: Vec<u8>) -> &mut Self {
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
        }
        self
    }

    pub fn set_recipient(&'a mut self, new_to: &'a str) -> &'a mut Self {
        match self {
            Message::Custom { ref mut to, .. } => {
                *to = Some(Address::new(new_to));
            }
            Message::Internal { ref mut to, .. } => {
                *to = Some(Address::new(new_to));
            }
        }
        self
    }
    pub fn uturn_with_reply(&'a mut self, reply: Option<Vec<u8>>) -> &'a mut Self {
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
        }
        self
    }

    pub fn with_recipients(&'a mut self, new_recipients: Vec<&'a str>) -> &'a mut Self {
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
        }
        self
    }

    pub fn set_recipient_all(&mut self) -> &mut Self {
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
        }
        self
    }

    pub fn get_content(&self) -> &Option<Vec<u8>> {
        match self {
            Message::Custom { content, .. } => content,
            Message::Internal { content, .. } => content,
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
        }
    }

    pub fn get_to(&'a self) -> &'a Option<Address<'a>> {
        match self {
            Message::Custom { to, .. } => to,
            Message::Internal { to, .. } => to,
        }
    }

    pub fn get_from(&'a self) -> &'a Option<Address<'a>> {
        match self {
            Message::Custom { from, .. } => from,
            Message::Internal { from, .. } => from,
        }
    }

    pub fn get_recipients(&'a self) -> &'a Option<AdditionalRecipients<'a>> {
        match self {
            Message::Custom { recipients, .. } => recipients,
            Message::Internal { recipients, .. } => recipients,
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
        }
    }

    pub(crate) fn internal(content: Option<Vec<u8>>, from: &'a str, to: &'a str) -> Self {
        Self::Internal {
            from: Some(Address::new(from)),
            to: Some(Address::new(to)),
            content,
            recipients: None,
            created: SystemTime::now(),
        }
    }
}

impl<'a> Message<'a> {
    pub async fn write<W: Seek + Write>(&'a self, w: &mut W) -> Result<()> {
        serde_json::to_writer(w, self)?;
        Ok(())
    }
}
impl<'a> Message<'a> {
    pub fn write_sync<W: Seek + Write>(&'a self, w: &mut W) -> Result<()> {
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
        let msg = msg.with_recipients(additional_recipients);
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
        let msg = msg.set_recipient("The new recipient");
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
        let msg = msg.uturn_with_reply(option_of_bytes("Reply"));
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
}
