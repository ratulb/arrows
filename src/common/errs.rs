#[derive(Debug)]
pub enum Error {
    Registration(std::io::Error),
    SerdeJson(serde_json::Error),
    Other(Box<dyn std::error::Error>),
    RegistrationError,
    RestorationError,
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
pub type Result<T> = std::result::Result<T, self::Error>;
