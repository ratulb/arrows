use crate::Address;
use serde::{Deserialize, Serialize};
use std::io::{Result, Seek, Write};
use std::time::SystemTime;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Message<T> {
    Custom {
        from: Option<Address>,
        to: Option<Address>,
        content: Option<T>,
        recipients: Option<AdditionalRecipients>,
        created: SystemTime,
    },
    Internal {
        from: Option<Address>,
        to: Option<Address>,
        content: Option<T>,
        recipients: Option<AdditionalRecipients>,
        created: SystemTime,
    },
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum AdditionalRecipients {
    All,
    OnlySome(Vec<Address>),
}

impl<T> Message<T> {
    pub fn new(content: T, from: &str, to: &str) -> Self {
        Self::Custom {
            from: Some(Address::new(from)),
            to: Some(Address::new(to)),
            content: Some(content),
            recipients: None,
            created: SystemTime::now(),
        }
    }

    pub fn with_content_and_to(&mut self, new_content: T, new_to: &str) -> &mut Self {
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
        self
    }

    pub fn set_recipient(&mut self, new_to: &str) -> &mut Self {
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

    pub fn with_recipients(&mut self, new_recipients: Vec<&str>) -> &mut Self {
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

    pub fn get_content(&self) -> &Option<T> {
        match self {
            Message::Custom { content, .. } => &content,
            Message::Internal { content, .. } => &content,
        }
    }

    pub fn get_to(&self) -> &Option<Address> {
        match self {
            Message::Custom { to, .. } => &to,
            Message::Internal { to, .. } => &to,
        }
    }

    pub fn get_from(&self) -> &Option<Address> {
        match self {
            Message::Custom { from, .. } => &from,
            Message::Internal { from, .. } => &from,
        }
    }

    pub fn get_recipients(&self) -> &Option<AdditionalRecipients> {
        match self {
            Message::Custom { recipients, .. } => &recipients,
            Message::Internal { recipients, .. } => &recipients,
        }
    }

    pub fn is_recipient_all(&self) -> bool {
        match self {
            Message::Custom { ref recipients, .. } => match recipients {
                &Some(AdditionalRecipients::All) => return true,
                _ => return false,
            },
            Message::Internal { ref recipients, .. } => match recipients {
                &Some(AdditionalRecipients::All) => return true,
                _ => return false,
            },
        }
    }

    pub(crate) fn internal(content: T, from: &str, to: &str) -> Self {
        Self::Internal {
            from: Some(Address::new(from)),
            to: Some(Address::new(to)),
            content: Some(content),
            recipients: None,
            created: SystemTime::now(),
        }
    }
}

impl<T: Serialize> Message<T> {
    pub async fn write<W: Seek + Write>(&self, w: &mut W) -> Result<()> {
        serde_json::to_writer(w, self)?;
        Ok(())
    }
}
impl<T: Serialize> Message<T> {
    pub fn write_sync<W: Seek + Write>(&self, w: &mut W) -> Result<()> {
        serde_json::to_writer(w, self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_of;
    use std::fs::{self, File, OpenOptions};
    use std::io::{self, BufWriter, Result, Seek};
    #[test]
    fn create_custom_msg_test_content_and_to() {
        let msg = Message::new("Content", "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &Some("Content"));
        assert_eq!(msg.get_to(), &Some(Address::new("addr_to")));
    }

    #[test]
    fn create_internal_msg_test_content_and_to() {
        let mut msg = Message::internal("Content", "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &Some("Content"));
        assert_eq!(msg.get_to(), &Some(Address::new("addr_to")));
    }

    #[test]
    fn create_custom_msg_test_from() {
        let mut msg = Message::new("Content", "addr_from", "addr_to");
        assert_eq!(msg.get_from(), &Some(Address::new("addr_from")));
    }

    #[test]
    fn create_internal_msg_test_from() {
        let mut msg = Message::internal("Content", "addr_from", "addr_to");
        assert_eq!(msg.get_from(), &Some(Address::new("addr_from")));
    }

    #[test]
    fn create_custom_msg_test_alter_content_and_to() {
        let mut msg = Message::new("Content", "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &Some("Content"));
        assert_eq!(msg.get_to(), &Some(Address::new("addr_to")));
        msg.with_content_and_to("New content", "New_to");
        assert_eq!(msg.get_content(), &Some("New content"));
        assert_eq!(msg.get_to(), &Some(Address::new("New_to")));
    }

    #[test]
    fn create_internal_msg_test_alter_content_and_to() {
        let mut msg = Message::internal("Content", "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &Some("Content"));
        assert_eq!(msg.get_to(), &Some(Address::new("addr_to")));
        msg.with_content_and_to("New content", "New_to");
        assert_eq!(msg.get_content(), &Some("New content"));
        assert_eq!(msg.get_to(), &Some(Address::new("New_to")));
    }

    #[test]
    fn alter_additional_recipients_test_1() {
        let mut msg = Message::internal("Content", "addr_from", "addr_to");
        let additional_recipients = vec!["Recipient1", "Recipient2", "Recipient3"];
        msg.with_recipients(additional_recipients);
        let additional_recipients_returned = vec!["Recipient1", "Recipient2", "Recipient3"];
        if let Some(AdditionalRecipients::OnlySome(recipients)) = msg.get_recipients() {
            let mut index = 0;
            recipients.iter().for_each(|addr| {
                assert_eq!(&addr.get_name()[..], additional_recipients_returned[index]);
                index += 1;
            });
        } else {
            panic!("Failed for message test - additional recipients");
        }
    }

    #[test]
    fn set_recipients_all_test_1() {
        let mut msg = Message::internal("Content", "addr_from", "addr_to");
        assert_eq!(msg.get_recipients(), &None);
        msg.set_recipient_all();
        type_of(&msg.get_recipients());
        assert_eq!(msg.get_recipients(), &Some(AdditionalRecipients::All));
    }

    #[test]
    fn set_recipient_test_1() {
        let mut msg = Message::internal("Content", "addr_from", "addr_to");
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
            inner: inner,
            elems: vec![simple],
        };

        let mut msg = Message::internal(complex, "addr_from", "addr_to");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("msg.txt")
            .expect("Complex msg write failure");
        let mut bufwriter = BufWriter::new(file);
        assert_eq!(msg.write_sync(&mut bufwriter).expect("Should get ()"), ());
    }
}
