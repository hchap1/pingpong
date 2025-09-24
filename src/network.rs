use crate::error::Res;

use iroh::{Endpoint, NodeAddr};
use iroh::endpoint::{Connection, RecvStream, SendStream};

use tokio::task::JoinHandle;

const ALPN: &[u8] = b"hchap1/pingpong";

pub struct Network {
    endpoint: Endpoint,
    recv_handle: JoinHandle<()>,
    send_handle: JoinHandle<()>
}

async fn connection_manager(connection: Connection) {

}

async fn recv(recv: RecvStream) {

}

async fn send(send: SendStream) {

}

impl Network {
    pub async fn client(addr: NodeAddr) -> Res<Self> {
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;
        let connection = endpoint.connect(addr, ALPN).await?;
        let (send, recv) = connection.open_bi().await?;

        Ok(Self {
            endpoint,
            recv_handle: tokio::spawn(async ),
        })
    }
}
