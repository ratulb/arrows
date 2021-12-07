use crate::common::addr::Addr;
use crate::common::utils::{compute_hash, from_bytes, option_of_bytes};
use serde::{Deserialize, Serialize};
use std::io::{Result, Seek, Write};
use std::mem::{replace, swap};
use std::time::SystemTime;
use uuid::Uuid;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Msg {
    Custom {
        id: u64,
        from: Option<Addr>,
        to: Option<Addr>,
        content: Option<Vec<u8>>,
        recipients: Option<AdditionalRecipients>,
        created: SystemTime,
    },
    Internal {
        id: u64,
        from: Option<Addr>,
        to: Option<Addr>,
        content: Option<Vec<u8>>,
        recipients: Option<AdditionalRecipients>,
        created: SystemTime,
    },
    Blank,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum AdditionalRecipients {
    All,
    OnlySome(Vec<Addr>),
}

impl Msg {
    pub fn new(content: Option<Vec<u8>>, from: &str, to: &str) -> Self {
        Self::Custom {
            id: compute_hash(&Uuid::new_v4()),
            from: Some(Addr::new(from)),
            to: Some(Addr::new(to)),
            content,
            recipients: None,
            created: SystemTime::now(),
        }
    }
    pub fn new_with_text(content: &str, from: &str, to: &str) -> Self {
        Self::Custom {
            id: compute_hash(&Uuid::new_v4()),
            from: Some(Addr::new(from)),
            to: Some(Addr::new(to)),
            content: option_of_bytes(&String::from(content)),
            recipients: None,
            created: SystemTime::now(),
        }
    }
    pub fn new_internal(content: &str, from: &str, to: &str) -> Self {
        Self::Custom {
            id: compute_hash(&Uuid::new_v4()),
            from: Some(Addr::new(from)),
            to: Some(Addr::new(to)),
            content: option_of_bytes(&String::from(content)),
            recipients: None,
            created: SystemTime::now(),
        }
    }

    pub fn content_as_text(&self) -> Option<&str> {
        match self {
            Msg::Custom { ref content, .. } => match content {
                Some(ref value) => {
                    let text: crate::Result<&str> = from_bytes(value);
                    text.ok()
                }
                None => None,
            },
            Msg::Internal { ref content, .. } => match content {
                Some(ref value) => {
                    let text: crate::Result<&str> = from_bytes(value);
                    text.ok()
                }
                None => None,
            },
            Msg::Blank => None,
        }
    }
    pub fn get_bytes(&self) -> Vec<u8> {
        option_of_bytes(self).unwrap_or(vec![])
    }

    pub fn uturn_with_text(&mut self, reply: &str) {
        match self {
            Msg::Custom {
                ref mut from,
                ref mut to,
                ref mut content,
                ..
            } => {
                swap(from, to);
                let _ignore = replace(content, option_of_bytes(&String::from(reply)));
            }
            Msg::Internal {
                ref mut from,
                ref mut to,
                ref mut content,
                ..
            } => {
                swap(from, to);
                let _ignore = replace(content, option_of_bytes(&String::from(reply)));
            }
            Msg::Blank => (),
        }
    }

    pub fn update_text_content(&mut self, reply: &str) {
        match self {
            Msg::Custom {
                ref mut content, ..
            } => {
                let _ignore = replace(content, option_of_bytes(&String::from(reply)));
            }
            Msg::Internal {
                ref mut content, ..
            } => {
                let _ignore = replace(content, option_of_bytes(&String::from(reply)));
            }
            Msg::Blank => (),
        }
    }

    pub fn with_content_and_to(&mut self, new_content: Vec<u8>, new_to: &str) {
        match self {
            Msg::Custom {
                ref mut content,
                ref mut to,
                ..
            } => {
                *content = Some(new_content);
                *to = Some(Addr::new(new_to));
            }
            Msg::Internal {
                ref mut content,
                ref mut to,
                ..
            } => {
                *content = Some(new_content);
                *to = Some(Addr::new(new_to));
            }
            Msg::Blank => (),
        }
    }

    pub fn with_content(&mut self, new_content: Vec<u8>) {
        match self {
            Msg::Custom {
                ref mut content, ..
            } => {
                *content = Some(new_content);
            }
            Msg::Internal {
                ref mut content, ..
            } => {
                *content = Some(new_content);
            }
            Msg::Blank => (),
        }
    }

    pub fn set_recipient(&mut self, new_to: &str) {
        match self {
            Msg::Custom { ref mut to, .. } => {
                *to = Some(Addr::new(new_to));
            }
            Msg::Internal { ref mut to, .. } => {
                *to = Some(Addr::new(new_to));
            }
            Msg::Blank => (),
        }
    }
    pub fn set_recipient_ip(&mut self, new_to_ip: &str) {
        match self {
            Msg::Custom { to, .. } => match to {
                Some(ref mut addr) => addr.with_ip(new_to_ip),
                None => (),
            },
            Msg::Internal { to, .. } => match to {
                Some(ref mut addr) => addr.with_ip(new_to_ip),
                None => (),
            },
            Msg::Blank => (),
        }
    }
    pub fn set_recipient_port(&mut self, new_port: u16) {
        match self {
            Msg::Custom { to, .. } => match to {
                Some(ref mut addr) => addr.with_port(new_port),
                None => (),
            },
            Msg::Internal { to, .. } => match to {
                Some(ref mut addr) => addr.with_port(new_port),
                None => (),
            },
            Msg::Blank => (),
        }
    }

    pub fn uturn_with_reply(&mut self, reply: Option<Vec<u8>>) {
        match self {
            Msg::Custom {
                ref mut from,
                ref mut to,
                ref mut content,
                ..
            } => {
                swap(from, to);
                let _ignore = replace(content, reply);
            }
            Msg::Internal {
                ref mut from,
                ref mut to,
                ref mut content,
                ..
            } => {
                swap(from, to);
                let _ignore = replace(content, reply);
            }
            Msg::Blank => (),
        }
    }

    pub fn with_recipients(&mut self, new_recipients: Vec<&str>) {
        match self {
            Msg::Custom {
                ref mut recipients, ..
            } => {
                *recipients = Some(AdditionalRecipients::OnlySome(
                    new_recipients.iter().map(|each| Addr::new(each)).collect(),
                ));
            }
            Msg::Internal {
                ref mut recipients, ..
            } => {
                *recipients = Some(AdditionalRecipients::OnlySome(
                    new_recipients.iter().map(|each| Addr::new(each)).collect(),
                ));
            }
            Msg::Blank => (),
        }
    }

    pub fn set_recipient_all(&mut self) {
        match self {
            Msg::Custom {
                ref mut recipients, ..
            } => {
                *recipients = Some(AdditionalRecipients::All);
            }
            Msg::Internal {
                ref mut recipients, ..
            } => {
                *recipients = Some(AdditionalRecipients::All);
            }
            Msg::Blank => (),
        }
    }

    pub fn get_content(&self) -> &Option<Vec<u8>> {
        match self {
            Msg::Custom { content, .. } => content,
            Msg::Internal { content, .. } => content,
            Msg::Blank => &None,
        }
    }
    //Would take content out - leaving message content to a None
    pub fn get_content_out(&mut self) -> Option<Vec<u8>> {
        match self {
            Msg::Custom {
                ref mut content, ..
            } => content.take(),
            Msg::Internal {
                ref mut content, ..
            } => content.take(),
            Msg::Blank => None,
        }
    }

    pub fn get_to(&self) -> &Option<Addr> {
        match self {
            Msg::Custom { to, .. } => to,
            Msg::Internal { to, .. } => to,
            Msg::Blank => &None,
        }
    }
    pub fn get_id(&self) -> &u64 {
        match self {
            Msg::Custom { id, .. } => id,
            Msg::Internal { id, .. } => id,
            Msg::Blank => &0,
        }
    }

    pub fn id_as_string(&self) -> String {
        self.get_id().to_string()
    }

    //Callers would be better served by cloning the returned &u64
    pub fn get_to_id(&self) -> u64 {
        match self {
            Msg::Custom { to, .. } => match to {
                Some(ref addr) => addr.get_id(),
                None => 0,
            },
            Msg::Internal { to, .. } => match to {
                Some(ref addr) => addr.get_id(),
                None => 0,
            },
            Msg::Blank => 0,
        }
    }

    pub fn get_from(&self) -> &Option<Addr> {
        match self {
            Msg::Custom { from, .. } => from,
            Msg::Internal { from, .. } => from,
            Msg::Blank => &None,
        }
    }
    pub fn is_outbound(&self) -> bool {
        match self {
            Msg::Custom { to, .. } => match to {
                Some(ref addr) => !addr.is_local(),
                None => false,
            },
            Msg::Internal { to, .. } => match to {
                Some(ref addr) => !addr.is_local(),
                None => false,
            },
            Msg::Blank => false,
        }
    }

    pub fn get_recipients(&self) -> &Option<AdditionalRecipients> {
        match self {
            Msg::Custom { recipients, .. } => recipients,
            Msg::Internal { recipients, .. } => recipients,
            Msg::Blank => &None,
        }
    }

    pub fn is_recipient_all(&self) -> bool {
        match self {
            Msg::Custom { ref recipients, .. } => {
                matches!(*recipients, Some(AdditionalRecipients::All))
            }
            Msg::Internal { ref recipients, .. } => {
                matches!(*recipients, Some(AdditionalRecipients::All))
            }
            Msg::Blank => false,
        }
    }

    pub fn internal(content: Option<Vec<u8>>, from: &str, to: &str) -> Self {
        Self::Internal {
            id: compute_hash(&Uuid::new_v4()),
            from: Some(Addr::new(from)),
            to: Some(Addr::new(to)),
            content,
            recipients: None,
            created: SystemTime::now(),
        }
    }
}

impl Default for Msg {
    fn default() -> Self {
        Msg::Blank
    }
}

impl Msg {
    pub async fn write<W: Seek + Write>(&self, w: &mut W) -> Result<()> {
        serde_json::to_writer(w, self)?;
        Ok(())
    }
}
impl Msg {
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
    use std::fs::OpenOptions;
    use std::io::BufWriter;
    #[test]
    fn create_custom_msg_test_content_and_to() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(
            from_bytes(&msg.get_content_out().unwrap()).ok(),
            Some("Content")
        );
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_to")));
    }

    #[test]
    fn create_internal_msg_test_content_and_to() {
        let msg = Msg::internal(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(*msg.get_content(), option_of_bytes(&"Content"));
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_to")));
    }

    #[test]
    fn create_custom_msg_test_from() {
        let msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_from(), &Some(Addr::new("addr_from")));
    }

    #[test]
    fn create_internal_msg_test_from() {
        let msg = Msg::internal(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_from(), &Some(Addr::new("addr_from")));
    }

    #[test]
    fn create_custom_msg_test_alter_content_and_to() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &option_of_bytes(&"Content"));
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_to")));
        msg.with_content_and_to(option_of_bytes(&"New content").unwrap(), "New_to");
        assert_eq!(msg.get_content(), &option_of_bytes(&"New content"));
        assert_eq!(msg.get_to(), &Some(Addr::new("New_to")));
    }

    #[test]
    fn create_internal_msg_test_alter_content_and_to() {
        let mut msg = Msg::internal(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &option_of_bytes(&"Content"));
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_to")));
        msg.with_content_and_to(option_of_bytes(&"New content").unwrap(), "New_to");
        assert_eq!(msg.get_content(), &option_of_bytes(&"New content"));
        assert_eq!(msg.get_to(), &Some(Addr::new("New_to")));
    }

    #[test]
    fn alter_additional_recipients_test_1() {
        let mut msg = Msg::internal(option_of_bytes(&"Content"), "addr_from", "addr_to");
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
        let mut msg = Msg::internal(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_recipients(), &None);
        msg.set_recipient_all();
        assert_eq!(msg.get_recipients(), &Some(AdditionalRecipients::All));
    }

    #[test]
    fn set_recipient_test_1() {
        let mut msg = Msg::internal(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_to")));
        msg.set_recipient("The new recipient");
        assert_eq!(msg.get_to(), &Some(Addr::new("The new recipient")));
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

        let msg = Msg::internal(option_of_bytes(&complex), "addr_from", "addr_to");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("msg.txt")
            .expect("Complex msg write failure");
        let mut bufwriter = BufWriter::new(file);
        assert!(msg.write_sync(&mut bufwriter).is_ok());
    }

    #[test]
    fn uturn_with_reply_test_1() {
        let mut msg = Msg::internal(option_of_bytes(&"Content"), "addr_from", "addr_to");
        msg.uturn_with_reply(option_of_bytes(&"Reply"));
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_from")));
        assert_eq!(msg.get_from(), &Some(Addr::new("addr_to")));
        assert_eq!(msg.get_content(), &option_of_bytes(&"Reply"));
    }
    #[test]
    fn uturn_with_text_reply_test_1() {
        let mut msg = Msg::internal(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &option_of_bytes(&"Content"));
        msg.uturn_with_text("Reply");
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_from")));
        assert_eq!(msg.get_from(), &Some(Addr::new("addr_to")));
        assert_eq!(msg.get_content(), &option_of_bytes(&"Reply"));
    }
    #[test]
    fn content_as_text_test_1() {
        let mut msg = Msg::internal(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &option_of_bytes(&"Content"));
        assert_eq!(msg.content_as_text(), Some("Content"));

        msg.update_text_content("Updated content");
        assert_eq!(msg.get_content(), &option_of_bytes(&"Updated content"));
        assert_eq!(msg.content_as_text(), Some("Updated content"));

        let mut msg = Msg::new(option_of_bytes(&"Custom content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), &option_of_bytes(&"Custom content"));
        assert_eq!(msg.content_as_text(), Some("Custom content"));

        msg.update_text_content("New updated content");
        assert_eq!(msg.get_content(), &option_of_bytes(&"New updated content"));
        assert_eq!(msg.content_as_text(), Some("New updated content"));
    }
    #[test]
    fn outbound_mgs_test_1() {
        let internal_msg = Msg::internal(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert!(!internal_msg.is_outbound());
        let custom_msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert!(!custom_msg.is_outbound());
        let blank = Msg::Blank;
        assert!(!blank.is_outbound());

        let mut internal_msg = Msg::internal(option_of_bytes(&"Content"), "addr_from", "addr_to");
        internal_msg.set_recipient_ip("89.89.89.89");

        assert!(internal_msg.is_outbound());
        let mut custom_msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");

        custom_msg.set_recipient_ip("89.89.89.89");
        assert!(custom_msg.is_outbound());
        let blank = Msg::Blank;
        assert!(!blank.is_outbound());
    }
}
