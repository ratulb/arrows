use crate::Result;
use crate::{
    common::utils::{compute_hash, from_bytes, option_of_bytes},
    Addr,
};

use serde::{Deserialize, Serialize};
use std::mem::{replace, swap};
use std::time::SystemTime;
use uuid::Uuid;
///The variants of actual message payload - Text, Binary blob or a Command adjoining an
///Action

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
            cont @ Self::Continue => write!(f, "{}", cont.as_text()),
            Self::Echo(s) => write!(f, "Echo({})", s),
        }
    }
}

use Content::*;
///The Mail enum which could be Trade(single message), Bulk(multiple messages) or Blank
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Mail {
    ///A mail variant with a single message inside. Actor's receives this variant
    Trade(Msg),
    ///Contains multiple messages - used for buffering, single shot transmission over the wire
    Bulk(Vec<Msg>),
    ///An empty mail
    Blank,
}
use Mail::*;

impl Mail {
    ///Get a handle to the inner message
    pub(crate) fn message(&self) -> &Msg {
        match self {
            Trade(ref msg) => msg,
            _ => panic!("message is supported only on Trade variant"),
        }
    }
    ///Get a handle to inner messages without hollowing out the mail
    pub fn messages(&self) -> &Vec<Msg> {
        match self {
            Bulk(ref msgs) => msgs,
            _ => panic!("messages is supported only on Bulk variant"),
        }
    }
    ///Take the inner message out - if its a Trade variant
    pub fn take(self) -> Msg {
        match self {
            Trade(msg) => msg,
            _ => panic!(),
        }
    }
    ///Take all the content out if its Mail enum variant is Bulk
    pub fn take_all(self) -> Vec<Msg> {
        match self {
            Bulk(msgs) => msgs,
            _ => panic!(),
        }
    }
    ///If the mail is actually a command - does it match a specific command
    pub fn command_equals(&self, action: Action) -> bool {
        if !self.is_command() {
            return false;
        }
        match self {
            Trade(_) => self.message().command_equals(action),
            bulk @ Bulk(_) => bulk.messages()[0].command_equals(action),
            _ => false,
        }
    }
    ///Is the mail is actually a containing a single command like Shutdown etc?
    pub fn is_command(&self) -> bool {
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

    ///Get the embedded [Action](arrows::Action) out if this mail is a command
    ///
    pub fn action(&self) -> Option<Action> {
        if !self.is_command() {
            return None;
        }
        match self {
            Trade(msg) => msg.action(),
            bulk @ Bulk(_) => bulk.messages()[0].action(),
            _ => None,
        }
    }

    //Actors are whimsical - might not respond immediately
    //Convert the buffered responses into single whole mail
    pub(crate) fn fold(mails: Vec<Option<Mail>>) -> Mail {
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
    ///Is the mail empty - mostly to avoid transmitting
    pub fn is_blank(mail: &Mail) -> bool {
        matches!(mail, Blank)
    }
    //Checks only for the variant of Trade!
    pub(crate) fn inbound(mail: &Mail) -> bool {
        match mail {
            Trade(ref msg) => msg.inbound(),
            _ => false,
        }
    }
    //Split into inbound and outbound(local vs remote)
    pub(crate) fn split(mail: Mail) -> Option<(Vec<Msg>, Vec<Msg>)> {
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
    pub(crate) fn partition(mails: Vec<Option<Mail>>) -> Option<(Vec<Msg>, Vec<Msg>)> {
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
    ///Set from address on all the messagess inside a potential mail
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
///[Msg](arrows::Msg) content type can also be [Command](arrows::Content::Command). `Action`
///represents tasks corresponding to Commands.
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Action {
    ///Shutdown the message listener
    Shutdown,
    ///Send an echo message to the listener
    Echo(String),
    ///Good to go, carry on
    Continue,
}
impl Action {
    //Meant for internal use by the system
    fn as_text(&self) -> &str {
        match self {
            Self::Shutdown => "Shutdown",
            Self::Echo(_) => "Echo",
            Self::Continue => "Continue",
        }
    }
    ///Inner content such as echo message
    pub fn inner(&self) -> &str {
        match self {
            Self::Shutdown => "",
            Self::Continue => "",
            Self::Echo(s) => s,
        }
    }

    ///Execute the action embedded in a Mail whose content  might be a Command
    ///For echo action execute just reverses the incoming text
    pub fn execute(&mut self, _input: Mail) -> Option<Mail> {
        match self {
            Self::Echo(text) => Some(Msg::echo(&text.chars().rev().collect::<String>()).into()),
            _ => None,
        }
    }
}
///The actual payload received by actors inside a Mail enum construct
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Default)]
pub struct Msg {
    id: u64,
    from: Option<Addr>,
    to: Option<Addr>,
    content: Option<Content>,
    dispatched: Option<SystemTime>,
}

impl Msg {
    ///Construct a new message with binary content with from and to addresses
    pub fn new(content: Option<Vec<u8>>, from: &str, to: &str) -> Self {
        Self {
            id: compute_hash(&Uuid::new_v4()),
            from: Some(Addr::new(from)),
            to: Some(Addr::new(to)),
            content: content.map(Binary),
            dispatched: None,
        }
    }
    ///Update the message with binary content
    pub fn with_binary_content(content: Option<Vec<u8>>) -> Self {
        Self {
            id: compute_hash(&Uuid::new_v4()),
            from: None,
            to: None,
            content: content.map(Binary),
            dispatched: None,
        }
    }

    ///Create a msg with content as text to an actor(`actor1`) in the local system
    ///
    ///
    /// # Example
    ///
    ///```
    ///use arrows::send;
    ///use arrows::Msg;
    ///
    ///let m = Msg::with_text("A good will message");
    ///send!("actor1", m);
    ///
    ///
    pub fn from_text(content: &str) -> Self {
        Self {
            id: compute_hash(&Uuid::new_v4()),
            from: None,
            to: None,
            content: Some(Text(content.to_string())),
            dispatched: Some(SystemTime::now()),
        }
    }
    ///Construct a text message with from and to addresses
    pub fn with_text(content: &str, from: &str, to: &str) -> Self {
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
    pub fn as_text(&self) -> Option<&str> {
        match self.content {
            Some(Binary(ref bytes)) => {
                let text: Result<&str> = from_bytes(bytes);
                text.ok()
            }
            Some(Text(ref s)) => Some(s),
            Some(Command(ref action)) => Some(action.inner()),
            None => None,
        }
    }
    ///Is the message actually a command?
    pub fn is_command(&self) -> bool {
        matches!(self.content, Some(Command(_)))
    }
    ///Command action equality check
    pub fn command_equals(&self, action: Action) -> bool {
        if !self.is_command() {
            return false;
        }
        if let Some(Content::Command(ref own_action)) = self.content {
            return own_action.as_text() == action.as_text();
        }
        false
    }
    ///Get the embedded [Action](arrows::Action) out if this [Msg](arrows::Msg) content
    ///is really is Command
    pub fn action(&self) -> Option<Action> {
        if !self.is_command() {
            return None;
        }
        match self.content {
            Some(Command(ref action)) => Some(action.clone()),
            _ => None,
        }
    }

    ///The message as bytes - irrespective of whether content is text or
    ///actual binary blob. Empty byte vec - if can not be serialized
    pub fn as_bytes(&self) -> Vec<u8> {
        option_of_bytes(self).unwrap_or_default()
    }
    ///Construct a text reply with content as string and message direction reversed
    pub fn text_reply(&mut self, reply: &str) {
        swap(&mut self.from, &mut self.to);
        let _ignore = replace(&mut self.content, Some(Text(reply.to_string())));
    }

    ///Update the content of the message - text
    pub fn update_text_content(&mut self, reply: &str) {
        let _ignore = replace(&mut self.content, Some(Text(reply.to_string())));
    }
    ///Set new binary content and new local recipient actor address
    pub fn with_content_and_to(&mut self, new_content: Vec<u8>, new_to: &str) {
        self.content = Some(Binary(new_content));
        self.to = Some(Addr::new(new_to));
    }
    ///Set the binary content of the message
    pub fn with_content(&mut self, new_content: Vec<u8>) {
        self.content = Some(Binary(new_content));
    }
    ///Set the recipient address of the message
    pub fn set_recipient_addr(&mut self, addr: &Addr) {
        self.to = Some(addr.clone());
    }
    ///Set the recipient identifier as string literal
    pub fn set_recipient(&mut self, new_to: &str) {
        self.to = Some(Addr::new(new_to));
    }
    ///Set the recipient actor's IP - used in remoting
    pub fn set_recipient_ip(&mut self, new_to_ip: &str) {
        if let Some(ref mut addr) = self.to {
            addr.with_ip(new_to_ip);
        }
    }
    ///Set the recipient port
    pub fn set_recipient_port(&mut self, new_port: u16) {
        if let Some(ref mut addr) = self.to {
            addr.with_port(new_port);
        }
    }
    ///Get the port of the recipient actor - used for remote messaging
    pub fn get_recipient_port(&self) -> u16 {
        if let Some(addr) = &self.to {
            return addr.get_port().expect("port");
        }
        0
    }
    ///Reverse the message direction - 'to' to 'from' or other way
    pub fn binary_reply(&mut self, reply: Option<Vec<u8>>) {
        swap(&mut self.from, &mut self.to);
        let _ignore = replace(&mut self.content, reply.map(Binary));
    }
    ///Get the binary content - the message remains intact
    pub fn binary_content(&self) -> Option<Vec<u8>> {
        match &self.content {
            Some(Binary(data)) => Some(data.to_vec()),
            _ => None,
        }
    }
    ///If the message content is binary blob - get it
    ///Would take content out - leaving message content to a None
    pub fn binary_content_out(&mut self) -> Option<Vec<u8>> {
        match self.content.take() {
            Some(Binary(data)) => Some(data),
            content @ Some(_) => {
                self.content = content;
                None
            }
            _ => None,
        }
    }
    ///Get the address of the actor the message is directed at
    pub fn get_to(&self) -> &Option<Addr> {
        &self.to
    }
    ///Get the id
    pub fn get_id(&self) -> &u64 {
        &self.id
    }
    ///Get the id as string. Required because unique ids overflow i64 range supported by
    ///the backing store
    pub fn id_as_string(&self) -> String {
        self.get_id().to_string()
    }

    ///Get the unique id of the actor message is directed at
    pub fn get_to_id(&self) -> u64 {
        match self.to {
            Some(ref addr) => addr.get_id(),
            None => 0,
        }
    }

    ///Get the from address
    pub fn get_from(&self) -> &Option<Addr> {
        &self.from
    }
    ///Check if message is directed at any actor in the system
    pub fn inbound(&self) -> bool {
        match self.to {
            Some(ref addr) => addr.is_local(),
            None => false,
        }
    }
    ///Set the from address of a message - specific usage while sending out actor message
    ///processing outcome
    pub fn set_from(&mut self, from: &Addr) {
        let _ignore = std::mem::replace(&mut self.from, Some(from.clone()));
    }

    ///Construct a Shutdown command to shutdown the system listener
    pub fn shutdown() -> Self {
        let mut cmd = Msg::default();
        let _ignore = std::mem::replace(&mut cmd.content, Some(Content::Command(Action::Shutdown)));
        cmd
    }
    ///Construct a Echo command to send to the system listener to check its liveness
    pub fn echo(s: &str) -> Self {
        let mut cmd = Msg::default();
        let _ignore = std::mem::replace(
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
            let _rs = match self.from {
                Some(ref from) => write!(f, "from: {}, ", from),
                None => write!(f, "from: None, "),
            };
            let _rs = match self.to {
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
                        let _ = write!(f, "{}", msgs[i]);
                        let _rs = writeln!(f);
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
            from_bytes(&msg.binary_content_out().unwrap()).ok(),
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
        assert_eq!(msg.binary_content(), option_of_bytes(&"Content"));
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_to")));
        msg.with_content_and_to(option_of_bytes(&"New content").unwrap(), "New_to");
        assert_eq!(msg.binary_content(), option_of_bytes(&"New content"));
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
    fn binary_reply_test_1() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        msg.binary_reply(option_of_bytes(&"Reply"));
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_from")));
        assert_eq!(msg.get_from(), &Some(Addr::new("addr_to")));
        assert_eq!(msg.binary_content(), option_of_bytes(&"Reply"));
    }
    #[test]
    fn text_reply_test_1() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.binary_content(), option_of_bytes(&"Content"));
        msg.text_reply("Reply");
        assert_eq!(msg.get_to(), &Some(Addr::new("addr_from")));
        assert_eq!(msg.get_from(), &Some(Addr::new("addr_to")));
        assert_eq!(msg.binary_content(), option_of_bytes(&"Reply"));
    }
    #[test]
    fn as_text_test_1() {
        let mut msg = Msg::new(option_of_bytes(&"Content"), "addr_from", "addr_to");
        assert_eq!(msg.binary_content(), option_of_bytes(&"Content"));
        assert_eq!(msg.as_text(), Some("Content"));

        msg.update_text_content("Updated content");
        assert_eq!(msg.binary_content(), option_of_bytes(&"Updated content"));
        assert_eq!(msg.as_text(), Some("Updated content"));
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
        let mut mails = vec![
            Some(Mail::Blank),
            Some(Mail::Blank),
            None,
            Some(Trade(Msg::with_text("mail", "from", "to"))),
        ];

        let mut m1 = Msg::with_text("mail", "from1", "to1");
        let mut addr1 = Addr::new("add1");
        addr1.with_port(9999);
        m1.set_recipient_addr(&addr1);
        mails.push(Some(Trade(m1)));
        let mut addr2 = addr1.clone();
        addr2.with_port(1111);
        let mut m2 = Msg::with_text("mail", "from2", "to2");
        m2.set_recipient_addr(&addr2);
        mails.push(Some(Bulk(vec![m2])));

        let mut m3 = Msg::with_text("mail", "from3", "to3");
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
        let m = Msg::with_text("mail", "from", "to");
        let mail = Mail::Trade(m);
        println!("{}", mail);

        let m1 = Msg::with_text("mail", "from", "to");
        let m2 = Msg::with_text("mail", "from", "to");
        let m3 = Msg::with_text("mail", "from", "to");
        let bulk_mail = Mail::Bulk(vec![]);
        println!("{}", bulk_mail);

        let bulk_mail = Mail::Bulk(Vec::new());
        println!("{}", bulk_mail);

        let bulk_mail = Mail::Bulk(vec![m1, m2, m3]);
        println!("Bulk {}", bulk_mail);

        let m1 = Msg::with_text("mail", "from", "to");
        let bulk_mail = Mail::Bulk(vec![m1]);
        println!("Bulk {}", bulk_mail);

        println!("Bulk {}", Mail::Blank);
    }

    #[test]
    fn mail_is_command_test_1() {
        let trade_mail: Mail = Msg::with_text("Some text", "from", "to").into();
        assert!(!trade_mail.is_command());

        let bulk = Mail::Bulk(vec![Msg::with_text("Some text", "from", "to")]);
        assert!(!bulk.is_command());
    }
}
