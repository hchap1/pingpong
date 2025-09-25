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

        let packet = server.get_next_message().await.unwrap();
        println!("{:?} SENT {:?}", packet.author, packet.content);


    } else if args[1] == "client" {

        let addr = match args.get(2) {
            Some(addr) => addr,
            None => {
                eprintln!("Client requires server token.");
                return
            }
        };

        let mut client = ForeignNodeContact::client(NodeAddr::new(NodeId::from_str(addr).unwrap())).await.unwrap();
        client.send(b"Hello, world!".to_vec()).await;

    }
}
