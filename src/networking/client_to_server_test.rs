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
                println!("{:?} SENT UTF-8 {:?}", packet.author, bytes);
                if let Ok(text) = String::from_utf8(bytes) {
                    println!("TRANSLATES TO {:?}", text);
                } else {
                    println!("IS NOT VALID UTF-8");
                }
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

        let message = args[2..].join(" ");

        let mut client = ForeignNodeContact::client(NodeAddr::new(NodeId::from_str(addr).unwrap())).await.unwrap();
        println!("SEND RESULT: {:?}", client.send(message.into_bytes()).await);
    }
}
