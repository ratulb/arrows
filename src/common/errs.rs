/// #Error
/// Various error messages during the arrows execution path.
#[derive(Debug)]
pub enum Error {
    ///IO erros during registration
    Registration(std::io::Error),
    ///Error while sending and receiving messages
    MsgSendError(std::io::Error),
    ///Errors related serde serialization and deserialization
    SerdeJson(serde_json::Error),
    ///Akin to unknown error
    Other(Box<dyn std::error::Error>),
    ///Invalid payload etc
    InvalidData,
    ///Registration error not due to IO
    RegistrationError,
    ///Error that might happen during actor activation from the backing store
    RestorationError,
    ///Most of the tings gets stored as binary blobs and system depends heavily depends on
    ///bincode - this variant captures errors related bincode
    Bincode(bincode::ErrorKind),
}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Registration(err)
    }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::SerdeJson(err)
    }
}

impl From<bincode::ErrorKind> for Error {
    fn from(err: bincode::ErrorKind) -> Error {
        Error::Bincode(err)
    }
}
impl From<Error> for std::io::Error {
    fn from(_err: Error) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, "Arrows error")
    }
}
///Success cases mostly related to the Mail enum, error cases are this crates' exposed erros
pub type Result<T> = std::result::Result<T, self::Error>;
