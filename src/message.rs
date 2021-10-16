use crate::Address;
use crate::Addresses;
use serde::{Deserialize, Serialize};
use std::io::{Result, Seek, Write};
use std::time::SystemTime;

//#[derive(Clone, Debug, Serialize, Deserialize)]
#[derive(Clone, Debug, Serialize)]
pub enum Message<T> {
    Business {
        from: Option<Address>,
        to: Option<Address>,
        content: Option<T>,
        created: SystemTime,
        signature: Option<Signature>,
        addressing: AddressMode,
        #[serde(skip_deserializing)]
        additional_recipients: Option<Addresses>,
    },
    System {
        from: Option<Address>,
        to: Option<Address>,
        content: Option<T>,
        created: SystemTime,
        signature: Option<Signature>,
        addressing: AddressMode,
        #[serde(skip_deserializing)]
        additional_recipients: Option<Addresses>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Signature;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AddressMode {
    OneToOne,
    OneToMany,
}
impl Default for AddressMode {
    fn default() -> Self {
        Self::OneToOne
    }
}

impl<T: for<'de> Deserialize<'de> + Clone + std::fmt::Debug + Serialize> Message<T> {
    pub fn new(content: T, from: &str, to: &str) -> Self {
        Self::Business {
            from: Some(Address::new(from)),
            to: Some(Address::new(to)),
            content: Some(content),
            created: SystemTime::now(),
            signature: None,
            addressing: AddressMode::default(),
            additional_recipients: None,
        }
    }
    pub(crate) fn system(content: T, from: &str, to: &str) -> Self {
        //TODO make this private
        Self::System {
            from: Some(Address::new(from)),
            to: Some(Address::new(to)),
            content: Some(content),
            created: SystemTime::now(),
            signature: None,
            addressing: AddressMode::default(),
            additional_recipients: None,
        }
    }
    pub fn with_recipient(&mut self, recipient: &str) -> &Self {
      if let Message::Business {ref mut to, ..} = self {
         *to =  Some(Address::new(recipient));
      }
      self
    }
}

impl<T: Serialize> Message<T> {
    pub async fn write<W: Seek + Write>(&self, w: &mut W) -> Result<()> {
        serde_json::to_writer(w, self)?;
        Ok(())
    }
}
