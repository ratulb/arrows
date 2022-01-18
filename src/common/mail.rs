use crate::Result;
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
    Command(Action),
}
impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            action @ Self::Shutdown => write!(f, "{}", action.as_text()),
            Self::Echo(s) => write!(f, "Echo({})", s),
        }
    }
}

use Content::*;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Mail {
    Trade(Msg),
    Bulk(Vec<Msg>),
    Blank,
}
use Mail::*;

impl Mail {
    pub fn message(&self) -> &Msg {
        println!("Inside mail message - printing self {}", self);
        match self {
            Trade(ref msg) => msg,
            _ => panic!("message is supported only on Trade variant"),
        }
    }

    pub fn messages(&self) -> &Vec<Msg> {
        match self {
            Bulk(ref msgs) => msgs,
            _ => panic!("messages is supported only on Bulk variant"),
        }
    }
    pub fn take(self) -> Msg {
        match self {
            Trade(msg) => msg,
            _ => panic!(),
        }
    }

    pub fn take_all(self) -> Vec<Msg> {
        match self {
            Bulk(msgs) => msgs,
            _ => panic!(),
        }
    }
    pub(crate) fn command_equals(&self, other: &Msg) -> bool {
        if !self.is_command() {
            return false;
        }
        match self {
            Trade(_) => self.message().command_equals(other),
            bulk @ Bulk(_) => bulk.messages()[0].command_equals(other),
            _ => false,
        }
    }

    pub(crate) fn is_command(&self) -> bool {
        match self {
            trade @ Trade(_) => trade.message().is_command(),
            bulk @ Bulk(ref _msgs)
                if bulk.messages().len() == 1 && bulk.messages()[0].is_command() =>
            {
                true
            }
            _ => false,
        }
    }

    pub fn fold(mails: Vec<Option<Mail>>) -> Mail {
        Bulk(
            mails
                .into_iter()
                .map(Mail::from)
                .flat_map(|mail| match mail {
                    trade @ Trade(_) => vec![trade.take()],
                    bulk @ Bulk(_) => bulk.take_all(),
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn is_blank(mail: &Mail) -> bool {
        match mail {
            Blank => true,
            _ => false,
        }
    }
    //Checks only for the variant of Trade!
    pub fn inbound(mail: &Mail) -> bool {
        match mail {
            Trade(ref msg) => msg.inbound(),
            _ => false,
        }
    }
    //Split into inbound and outbound(local vs remote)
    pub fn split(mail: Mail) -> Option<(Vec<Msg>, Vec<Msg>)> {
        match mail {
            Blank => None,
            trade @ Trade(_) if Mail::inbound(&trade) => Some((vec![trade.take()], Vec::new())),
            trade @ Trade(_) if !Mail::inbound(&trade) => Some((Vec::new(), vec![trade.take()])),
            Trade(_) => unreachable!(),
            Bulk(msgs) => match msgs.into_iter().partition::<Vec<Msg>, _>(Msg::inbound) {
                (v1, v2) if v1.is_empty() & v2.is_empty() => None,
                or_else @ (_, _) => Some(or_else),
            },
        }
    }

    //partition as inbound/outbound messages
    pub fn partition(mails: Vec<Option<Mail>>) -> Option<(Vec<Msg>, Vec<Msg>)> {
        match mails
            .into_iter()
            .map(Mail::from)
            .filter(|mail| !Mail::is_blank(mail))
            .flat_map(|mail| match mail {
                trade @ Trade(_) => vec![trade.take()],
                Bulk(msgs) => msgs,
                _ => panic!(),
            })
            .partition::<Vec<Msg>, _>(Msg::inbound)
        {
            (v1, v2) if v1.is_empty() & v2.is_empty() => None,
            or_else @ (_, _) => Some(or_else),
        }
    }

    pub fn set_from(mail: &mut Option<Mail>, from: &Addr) {
        match mail {
            Some(mail) => match mail {
                Trade(msg) => msg.set_from(from),
                Bulk(msgs) => {
                    for msg in msgs.iter_mut() {
                        msg.set_from(from);
                    }
                }
                Blank => (),
            },
            None => (),
        }
    }
}

impl From<Option<Mail>> for Mail {
    fn from(opt: Option<Mail>) -> Self {
        match opt {
            Some(mail) => mail,
            None => Mail::Blank,
        }
    }
}

impl From<Vec<Msg>> for Mail {
    fn from(msgs: Vec<Msg>) -> Self {
        Bulk(msgs)
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Action {
    Shutdown,
    Echo(String),
}
impl Action {
    fn as_text(&self) -> &str {
        match self {
            Self::Shutdown => "Shutdown",
            Self::Echo(_) => "Echo",
        }
    }
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
            content: content.map(Binary),
            dispatched: None,
        }
    }
    ///Create a msg with content as text to an actor(`example_actor1`) in the local system
    ///
    /// # Example
    ///
    ///```
    ///use arrows::send;
    ///use arrows::Msg;
    ///
    ///let m = Msg::from_text("mail", "from", "example_actor1");
    ///let rs = send!("example_actor1", m);
    ///
    ///

    pub fn from_text(content: &str, from: &str, to: &str) -> Self {
        Self {
            id: compute_hash(&Uuid::new_v4()),
            from: Some(Addr::new(from)),
            to: Some(Addr::new(to)),
            content: Some(Text(content.to_string())),
            dispatched: Some(SystemTime::now()),
        }
    }
    /// Get the content of msg as text. In case - binary content being actually binary
    /// this would not be helpful.
    pub fn content_as_text(&self) -> Option<&str> {
        match self.content {
            Some(Binary(ref bytes)) => {
                let text: Result<&str> = from_bytes(bytes);
                text.ok()
            }
            Some(Text(ref s)) => Some(s),
            Some(Command(ref cmd)) => Some(cmd.as_text()),
            None => None,
        }
    }

    pub fn is_command(&self) -> bool {
        match self.content {
            Some(Command(_)) => true,
            _ => false,
        }
    }

    pub fn command_equals(&self, other: &Msg) -> bool {
        self.is_command() && self.content == other.content
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        option_of_bytes(self).unwrap_or_default()
    }

    pub fn text_reply(&mut self, reply: &str) {
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

    pub fn set_recipient_addr(&mut self, addr: &Addr) {
        self.to = Some(addr.clone());
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

    pub fn get_recipient_port(&self) -> u16 {
        if let Some(addr) = &self.to {
            return addr.get_port().expect("port");
        }
        0
    }

    pub fn reply_with_binary_content(&mut self, reply: Option<Vec<u8>>) {
        swap(&mut self.from, &mut self.to);
        let _ignore = replace(&mut self.content, reply.map(Binary));
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

    pub fn get_to_id(&self) -> u64 {
        match self.to {
            Some(ref addr) => addr.get_id(),
            None => 0,
        }
    }

    pub fn get_from(&self) -> &Option<Addr> {
        &self.from
    }
    pub fn inbound(&self) -> bool {
        match self.to {
            Some(ref addr) => addr.is_local(),
            None => false,
        }
    }

    pub fn set_from(&mut self, from: &Addr) {
        std::mem::replace(&mut self.from, Some(from.clone()));
    }
    pub fn shutdown() -> Self {
        let mut cmd = Msg::default();
        std::mem::replace(&mut cmd.content, Some(Content::Command(Action::Shutdown)));
        cmd
    }

    pub fn echo(s: &str) -> Self {
        let mut cmd = Msg::default();
        std::mem::replace(
            &mut cmd.content,
            Some(Content::Command(Action::Echo(s.to_string()))),
        );
        cmd
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

impl std::fmt::Display for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        {
            write!(f, "Msg({}), ", &self.id)?;
            match self.from {
                Some(ref from) => write!(f, "from: {}, ", from),
                None => write!(f, "from: None, "),
            };
            match self.to {
                Some(ref to) => write!(f, "to: {}, ", to),
                None => write!(f, "to: None, "),
            };
            match self.content {
                Some(ref content) => write!(f, "content: {}", content),
                None => write!(f, "content: None"),
            }
        }
    }
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Text(text) => {
                write!(f, "Text({})", text)
            }
            Binary(binary) => {
                write!(f, "Binary(..) -> length {}", binary.len())
            }
            Command(c) => {
                write!(f, "Command({})", c)
            }
        }
    }
}

impl std::fmt::Display for Mail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Trade(msg) => {
                write!(f, "Trade({})", msg)
            }
            Bulk(ref msgs) => {
                writeln!(f, "Bulk({})", msgs.len())?;
                if !msgs.is_empty() {
                    for i in 0..msgs.len() - 1 {
                        write!(f, "{}", msgs[i]);
                        writeln!(f);
                    }
                    write!(f, "{}", msgs[msgs.len() - 1])
                } else {
                    write!(f, " Empty")
                }
            }
            Blank => write!(f, "Blank"),
        }
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
        println!("{}", msg);
    }

    #[test]
    fn create_trade_msg_test_from() {
        let msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_from(), &Some(Addr::new("addr_from")));
        println!("{}", msg);
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
    fn reply_with_binary_content_test_1() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        msg.reply_with_binary_content(option_of_bytes(&"Reply"));
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_from")));
        assert_eq!(msg.get_from(), &Some(Addr::new("addr_to")));
        assert_eq!(msg.get_content(), option_of_bytes(&"Reply"));
    }
    #[test]
    fn text_reply_test_1() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.get_content(), option_of_bytes(&"Content"));
        msg.text_reply("Reply");
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
        assert!(trade_msg.inbound());

        trade_msg.set_recipient_ip("89.89.89.89");
        assert!(!trade_msg.inbound());
    }
    #[test]
    fn test_mail_partition() {
        let mut mails = vec![];
        mails.push(Some(Mail::Blank));
        mails.push(Some(Mail::Blank));
        mails.push(None);
        mails.push(Some(Trade(Msg::from_text("mail", "from", "to"))));
        let mut m1 = Msg::from_text("mail", "from1", "to1");
        let mut addr1 = Addr::new("add1");
        addr1.with_port(9999);
        m1.set_recipient_addr(&addr1);
        mails.push(Some(Trade(m1)));
        let mut addr2 = addr1.clone();
        addr2.with_port(1111);
        let mut m2 = Msg::from_text("mail", "from2", "to2");
        m2.set_recipient_addr(&addr2);
        mails.push(Some(Bulk(vec![m2])));

        let mut m3 = Msg::from_text("mail", "from3", "to3");
        m3.set_recipient_ip("89.89.89.89");
        mails.push(Some(Bulk(vec![m3])));

        if let Some((ref v1, ref v2)) = Mail::partition(mails) {
            v1.iter().for_each(|msg| {
                if let Some(ref addr) = msg.get_to() {
                    assert!(addr.is_local());
                }
            });

            v2.iter().for_each(|msg| {
                if let Some(ref addr) = msg.get_to() {
                    assert!(!addr.is_local());
                }
            });
        }
    }

    #[test]
    fn mail_print_test() {
        let m = Msg::from_text("mail", "from", "to");
        let mail = Mail::Trade(m);
        println!("{}", mail);

        let m1 = Msg::from_text("mail", "from", "to");
        let m2 = Msg::from_text("mail", "from", "to");
        let m3 = Msg::from_text("mail", "from", "to");
        let bulk_mail = Mail::Bulk(vec![]);
        println!("{}", bulk_mail);

        let bulk_mail = Mail::Bulk(Vec::new());
        println!("{}", bulk_mail);

        let bulk_mail = Mail::Bulk(vec![m1, m2, m3]);
        println!("Bulk {}", bulk_mail);

        let m1 = Msg::from_text("mail", "from", "to");
        let bulk_mail = Mail::Bulk(vec![m1]);
        println!("Bulk {}", bulk_mail);

        println!("Bulk {}", Mail::Blank);
    }

    #[test]
    fn mail_is_command_test_1() {
        let trade_mail: Mail = Msg::from_text("Some text", "from", "to").into();
        assert!(!trade_mail.is_command());

        let bulk = Mail::Bulk(vec![Msg::from_text("Some text", "from", "to")]);
        assert!(!bulk.is_command());
    }
}
