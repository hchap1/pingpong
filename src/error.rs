use async_channel::RecvError;
use iroh::endpoint::{BindError, ConnectError, ConnectionError, RemoteNodeIdError};

pub type Res<T> = Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Error {
    MagicSpawn,
    Discovery,
    Connect,
    Connection(Option<ConnectionError>),
    Unknown,

    StreamClosed,
    StreamCrashed,
    StreamReadFailed,
    TooLong,

    RemoteIDFailed,

    MPMCRecvError
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
            ConnectError::Connection { source: _, backtrace: _, span_trace: _ } => Error::Connection(None),
            _ => Error::Unknown
        }
    }
}

impl From<ConnectionError> for Error {
    fn from(error: ConnectionError) -> Self {
        Error::Connection(Some(error))
    }
}

impl From<RemoteNodeIdError> for Error {
    fn from(_error: RemoteNodeIdError) -> Self {
        Error::RemoteIDFailed
    }
}

impl From<RecvError> for Error {
    fn from(_error: RecvError) -> Self {
        Self::MPMCRecvError
    }
}
