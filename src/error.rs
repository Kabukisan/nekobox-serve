use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    UserAlreadyExists,
    ValidationError(validator::ValidationError),
    SqliteError(rusqlite::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(value: rusqlite::Error) -> Self {
        Error::SqliteError(value)
    }
}

impl From<validator::ValidationError> for Error {
    fn from(value: validator::ValidationError) -> Self {
        Error::ValidationError(value)
    }
}
