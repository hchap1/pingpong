use crate::error::Res;

use iroh::{Endpoint, NodeAddr};
use iroh::endpoint::Connection;

const ALPN: &[u8] = b"hchap1/pingpong";

pub struct Network {
    endpoint: Endpoint,
    connection: Connection
}

impl Network {
    pub async fn client(addr: NodeAddr) -> Res<Self> {
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;
        let connection = endpoint.connect(addr, ALPN).await?;
        let (send, recv) = connection.open_bi().await?;

        Ok(Self {
            endpoint,
            connection
        })
    }
}
