#[derive(Debug)]
pub enum Error {
    Registration(std::io::Error),
    SerdeJson(serde_json::Error),
    Other(Box<dyn std::error::Error>),
    RegistrationError,
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

pub type Result<T> = std::result::Result<T, self::Error>;
