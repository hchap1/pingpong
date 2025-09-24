use iroh::endpoint::{BindError, ConnectError};

pub type Res<T> = Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Error {
    MagicSpawn,
    Discovery,
    Connect,
    Connection,
    Unknown
}

impl From<BindError> for Error {
    fn from(error: BindError) -> Self {
        match error {
            BindError::MagicSpawn { source: _, backtrace: _, span_trace: _ } => Error::MagicSpawn,
            BindError::Discovery { source: _, backtrace: _, span_trace: _ } => Error::Discovery,
            _ => Error::Unknown
        }
    }
}

impl From<ConnectError> for Error {
    fn from(error: ConnectError) -> Self {
        match error {
            ConnectError::Connect { source: _, backtrace: _, span_trace: _ } => Error::Connect,
            ConnectError::Connection { source: _, backtrace: _, span_trace: _ } => Error::Connection,
            _ => Error::Unknown
        }
    }
}
