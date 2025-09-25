mod network;
mod error;
mod packet;

use std::{env::args, str::FromStr};

use iroh::{NodeAddr, NodeId};
use network::{ForeignNodeContact, Server};

#[tokio::main]
async fn main() {

    let args: Vec<String> = args().collect::<Vec<String>>();

    if args.len() <= 1 {
        eprintln!("Requires either:\nserver\nclient <ADDR>");
        return;
    }

    if args[1] == "server" {
        
        let server = Server::spawn().await.unwrap();
        println!("SERVER ADDRESS: {}", server.get_address().node_id);

        while let Ok(packet) = server.get_next_message().await {
            if let Ok(bytes) = packet.content {
                println!("{:?} SENT {:?}", packet.author, String::from_utf8(bytes));
            } else {
                println!("{:?} DISCONNECT", packet.author);
            }
        }

    } else if args[1] == "client" {

        let addr = match args.get(2) {
            Some(addr) => addr,
            None => {
                eprintln!("Client requires server token.");
                return
            }
        };

        let message = match args.get(3) {
            Some(message) => message.clone(),
            None => {
                eprintln!("Client requires message to send.");
                return
            }
        };

        let mut client = ForeignNodeContact::client(NodeAddr::new(NodeId::from_str(addr).unwrap())).await.unwrap();
        println!("SEND RESULT: {:?}", client.send(message.into_bytes()).await);
    }
}
