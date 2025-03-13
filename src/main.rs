mod server;
use std::io::{stdin, Read, Write};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use crate::server::server::{Server, ServerCommand};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port = &args[1];

    let (tx, rx) = mpsc::channel();
    let mut server = Server::new(port);

    server.listen();
    server.read_messages();
    server.handle_commands(rx);

    loop {
        commands(tx.clone());
    }
}


fn commands(sender: Sender<ServerCommand>) {
    println!("1 - Connect");
    println!("2 - Send a message");

    let stdin = stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();

    match input.trim() {
        "1" => {
            println!("Port: ");
            let mut port = String::new();
            stdin.read_line(&mut port).unwrap();

            let port = port.trim();
            if let Ok(_) = port.parse::<u16>() {
                sender.send(ServerCommand::Connect(port.to_string())).unwrap();
            } else {
                println!("Invalid port input.");
            }
        }
        "2" => {
            println!("Type your message: ");

            let mut message = String::new();
            stdin.read_line(&mut message).unwrap();

            let message = message.trim();
            sender.send(ServerCommand::SendMessage(message.to_string())).unwrap();
        }
        _ => {
            println!("Invalid command");
        }
    }
}
