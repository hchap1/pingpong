use std::str::FromStr;

use iroh::NodeId;

use crate::{backend::database::{DatabaseParam, DatabaseParams}, error::{Error, Res}};

#[derive(Clone, Debug)]
pub struct Contact {
    server_address: NodeId,
    username: String
}

impl Contact {
    pub fn new(server: String, username: String) -> Res<Self> {
        match NodeId::from_str(&server) {
            Ok(server_address) => Ok(Self { server_address, username }),
            Err(_) => Err(Error::RemoteIDFailed)
        }
    }

    pub fn to_params(self) -> DatabaseParams {
        DatabaseParams::new(vec![
            DatabaseParam::String(self.server_address.to_string()),
            DatabaseParam::String(self.username)
        ])
    }
}
