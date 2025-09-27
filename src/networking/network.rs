use crate::error::{Error, Res};
use crate::networking::packet::Packet;

use iroh::protocol::{AcceptError, ProtocolHandler, Router};
use iroh::{Endpoint, NodeAddr, NodeId, Watcher};
use iroh::endpoint::{Connection, ReadToEndError, RecvStream};

use async_channel::Sender;
use async_channel::Receiver;
use async_channel::unbounded;

use super::packet::PacketType;

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
    _endpoint: Endpoint,
    connection: Connection
}

/// Consume bytes from recv stream and forward to relay stream.
async fn relay_bytes(foreign: NodeId, mut recv: RecvStream, relay: Sender<Packet>) {
    loop {
        println!("LISTENING TO CHANNEL FROM {foreign}");
        // Attempt to find packet to forward, else handle errors gracefully.
        let (forward, close) = match recv.read_to_end(4096).await {
            Ok(read) => {
                println!("RECEIVED: {read:?}");
                let empty = read.is_empty();
                (
                    if empty { vec![Packet::failure(foreign, Error::StreamClosed)] } else { Packet::success(foreign, read) },
                    empty
                )
            },
            Err(e) => {
                println!("CHANNEL FAILURE FROM {foreign}");
                (
                vec![match e {
                    ReadToEndError::Read(_) => Packet::failure(foreign, Error::StreamReadFailed),
                    ReadToEndError::TooLong => Packet::failure(foreign, Error::TooLong)
                }], true
            )}
        };

        // Closed stream (intentional / crash) results in termination of this thread.
        if close { break; }

        // Forward packet, terminating if relay channel is closed.
        for packet in forward {
            if relay.send(packet).await.is_err() { break; }
        }
    }
}

impl ForeignNodeContact {

    /// Establish a channel to a NodeAddr to send it packets.
    pub async fn client(addr: NodeId) -> Res<Self> {
        let endpoint = Endpoint::builder().discovery_n0().bind().await?;
        let connection = endpoint.connect(addr, ALPN).await?;

        Ok(Self {
            _endpoint: endpoint,
            connection,
        })
    }

    /// Encode the packet such that it can be split up using length headers
    pub async fn send(&mut self, mut packet: Vec<u8>, packet_type: PacketType) -> Res<()> {
        let len = packet.len() as u32;

        let mut header = Vec::with_capacity(5);
        header.push(packet_type.to_u8());
        header.extend_from_slice(&len.to_be_bytes());

        header.extend_from_slice(&packet);
        packet = header;

        let mut send_stream = self.connection.open_uni().await?;

        send_stream
            .write_all(&packet)
            .await .map_err(|_| Error::StreamClosed)?;

        if send_stream.finish().is_err() {
            Err(Error::StreamCrashed)
        } else {
            Ok(())
        }
    }
}

/// Receives and manages connections from foreign client nodes.
#[derive(Debug)]
pub struct Server {
    node_addr: NodeAddr,
    _router: Router,
    _send_stream: Sender<Packet>,
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
            _router: router,
            _send_stream: send_stream,
            recv_stream
        })
    }

    pub fn get_address(&self) -> NodeAddr {
        self.node_addr.clone()
    }

    pub fn yield_receiver(&self) -> Receiver<Packet> {
        self.recv_stream.clone()
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

        loop {
            let recv = connection.accept_uni().await?;

            // Relay until stream is closed by the other end.
            relay_bytes(node_id, recv, self.relay.clone()).await;

            println!("\nNEW UNI CHANNEL\n");
        }
    }
}
