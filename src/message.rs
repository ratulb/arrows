use crate::Address;
use serde::{Deserialize, Serialize};
use std::io::{Result, Seek, Write};
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message<T> {
    Custom {
        from: Option<Address>,
        to: Option<Address>,
        content: Option<T>,
        recipents: Option<AdditionalRecipents>,
        created: SystemTime,
    },
    Internal {
        from: Option<Address>,
        to: Option<Address>,
        content: Option<T>,
        recipents: Option<AdditionalRecipents>,
        created: SystemTime,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AdditionalRecipents {
    All,
    OnlySome(Vec<Address>),
}

impl<T> Message<T> {
    pub fn new(content: T, from: &str, to: &str) -> Self {
        Self::Custom {
            from: Some(Address::new(from)),
            to: Some(Address::new(to)),
            content: Some(content),
            recipents: None,
            created: SystemTime::now(),
        }
    }
    pub(crate) fn internal(content: T, from: &str, to: &str) -> Self {
        Self::Internal {
            from: Some(Address::new(from)),
            to: Some(Address::new(to)),
            content: Some(content),
            recipents: None,
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
