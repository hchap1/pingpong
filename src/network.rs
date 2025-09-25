use crate::error::{Error, Res};
use crate::packet::Packet;

use iroh::protocol::{AcceptError, ProtocolHandler, Router};
use iroh::{Endpoint, NodeAddr, NodeId, Watcher};
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

/// Local client connected to a foreign server.
#[derive(Debug)]
pub struct ForeignNodeContact {
    endpoint: Endpoint,
    connection: Connection,
    recv_handle: JoinHandle<()>,

    send_stream: SendStream,
    recv_stream: Receiver<Packet>
}

/// Consume bytes from recv stream and forward to relay stream.
async fn relay_bytes(foreign: NodeId, mut recv: RecvStream, relay: Sender<Packet>) {
    let mut buf: Vec<u8> = Vec::new();

    loop {
        // Attempt to find packet to forward, else handle errors gracefully.
        let (forward, close) = match recv.read(&mut buf).await {
            Ok(read) => (match read {
                Some(_bytes) => Packet::success(foreign.clone(), std::mem::take(&mut buf)),
                None => Packet::failure(foreign.clone(), Error::StreamReadFailed)
            }, false),
            Err(e) => (match e {
                ReadError::ClosedStream => Packet::failure(foreign.clone(), Error::StreamClosed),
                _ => Packet::failure(foreign.clone(), Error::StreamCrashed)
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
        let foreign = connection.remote_node_id()?;
        let (send, recv) = connection.open_bi().await?;
        let (relay, extractor) = unbounded();

        Ok(Self {
            endpoint,
            connection,

            // Spawn a relay for this, even though it is only one way by protocol.
            recv_handle: tokio::spawn(relay_bytes(foreign, recv, relay)),
            send_stream: send,
            recv_stream: extractor
        })
    }

    pub async fn send(&mut self, packet: Vec<u8>) {
        self.send_stream.write_all(&packet).await;
    }
}

/// Receives and manages connections from foreign client nodes.
#[derive(Debug)]
pub struct Server {
    node_addr: NodeAddr,
    router: Router,
    clients: Vec<ForeignNodeContact>,
    send_stream: Sender<Packet>,
    recv_stream: Receiver<Packet>
}

impl Server {
    
    /// Create a server
    pub async fn spawn() -> Res<Self> {

        let (send_stream, recv_stream) = unbounded();
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;
        let router = Router::builder(endpoint).accept(ALPN, PacketRelay { relay: send_stream.clone() }).spawn();

        Ok(Server {
            node_addr: router.endpoint().node_addr().initialized().await,
            router,
            clients: Vec::new(),
            send_stream, recv_stream
        })
    }

    pub fn get_address(&self) -> NodeAddr {
        self.node_addr.clone()
    }

    pub async fn get_next_message(&self) -> Res<Packet> {
        Ok(self.recv_stream.recv().await?)
    }
}

#[derive(Debug, Clone)]
pub struct PacketRelay {

    // Relay messages onto the server message stack
    relay: Sender<Packet>
}

impl ProtocolHandler for PacketRelay {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {

        let node_id = connection.remote_node_id()?;
        let (_send, recv) = connection.accept_bi().await?;

        // Relay until stream is closed by the other end.
        relay_bytes(node_id, recv, self.relay.clone()).await;

        Ok(())
    }
}
