use crate::error::{Error, Res};
use crate::packet::Packet;

use iroh::{Endpoint, NodeAddr};
use iroh::endpoint::{Connection, ReadError, RecvStream, SendStream};

use tokio::task::JoinHandle;

use async_channel::Sender;
use async_channel::Receiver;
use async_channel::unbounded;

const ALPN: &[u8] = b"hchap1/pingpong";

/* -- PROTOCOL --

 - Each node runs a server and a client.
 - Each node runs a server to accept messages.
 - When a node wishes to send a packet to another node, it must make a client to connect to the server of that node.
 - Channels, whilst bidirection, are used exclusively for CLIENT -> SERVER communication except for ending the channel.

*/

pub struct ForeignNodeContact {
    endpoint: Endpoint,
    connection: Connection,
    recv_handle: JoinHandle<()>,

    send_stream: SendStream,
    recv_stream: Receiver<Packet>
}

/// Consume bytes from recv stream and forward to relay stream.
async fn relay_bytes(addr: NodeAddr, mut recv: RecvStream, relay: Sender<Packet>) {
    let mut buf: Vec<u8> = Vec::new();

    loop {
        // Attempt to find packet to forward, else handle errors gracefully.
        let (forward, close) = match recv.read(&mut buf).await {
            Ok(read) => (match read {
                Some(_bytes) => Packet::success(addr.clone(), std::mem::take(&mut buf)),
                None => Packet::failure(addr.clone(), Error::StreamReadFailed)
            }, false),
            Err(e) => (match e {
                ReadError::ClosedStream => Packet::failure(addr.clone(), Error::StreamClosed),
                _ => Packet::failure(addr.clone(), Error::StreamCrashed)
            }, true)
        };

        // Closed stream (intentional / crash) results in termination of this thread.
        if close { break; }

        // Forward packet, terminating if relay channel is closed.
        if relay.send(forward).await.is_err() { break; }
    }
}

impl ForeignNodeContact {

    /// Establish a channel to a NodeAddr to send it packets.
    pub async fn client(addr: NodeAddr) -> Res<Self> {
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;
        let connection = endpoint.connect(addr.clone(), ALPN).await?;
        let (send, recv) = connection.open_bi().await?;
        let (relay, extractor) = unbounded();

        Ok(Self {
            endpoint,
            connection,

            // Spawn a relay for this, even though it is only one way by protocol.
            recv_handle: tokio::spawn(relay_bytes(addr, recv, relay)),
            send_stream: send,
            recv_stream: extractor
        })
    }
}
