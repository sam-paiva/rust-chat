use std::io::{BufRead, BufReader, Read};
use crate::server::client::Client;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

pub enum ServerCommand {
    Connect(String),
    SendMessage(String),
}

pub struct Server {
    listener: Arc<TcpListener>,
    clients: Arc<Mutex<Vec<Client>>>,
}

impl Server {
    pub fn new(port: &str) -> Server {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();

        Server {
            listener: Arc::new(listener),
            clients: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn handle_commands(&self, receiver: Receiver<ServerCommand>) {
        let temp_clients = Arc::clone(&self.clients);
        thread::spawn(move || {
            while let Ok(command) = receiver.recv() {
                match command {
                    ServerCommand::Connect(port) => {
                        let clients = temp_clients.lock().unwrap();
                        Server::connect(clients, port.as_str());
                    }
                    ServerCommand::SendMessage(message) => {
                        let clients = temp_clients.lock().unwrap();
                        Server::send_message(clients, message);
                    }
                }
            }
        });
    }

    fn send_message(mut clients: MutexGuard<Vec<Client>>, message: String) {
        for client in clients.iter_mut() {
            client.write_stream(message.as_str());
        }
    }

    fn connect(mut clients: MutexGuard<Vec<Client>>, port: &str) {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();

        let client = Client::new(stream.try_clone().unwrap());
        clients.push(client);
        println!("Connected successfully!");
    }

    pub fn read_messages(&mut self)  {
        let temp_client = Arc::clone(&self.clients);
        thread::spawn(move || {
            loop {
                let mut clients = temp_client.lock().unwrap();

                for client in clients.iter_mut() {
                    let data = client.read();

                    match data {
                        None => {}
                        Some(data) => {
                            println!("Received: {}", data);
                        }
                    }
                }
            }
        });
    }

    pub fn listen(&mut self) {
        let temp_listener = Arc::clone(&self.listener);
        let temp_clients = Arc::clone(&self.clients);

        thread::spawn(move || {
            for stream in temp_listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        println!("New connection: {}", stream.try_clone().unwrap().peer_addr().unwrap());

                        let client = Client::new(stream.try_clone().unwrap());
                        temp_clients.lock().unwrap().push(client);
                    }
                    _ => {}
                }
            }
        });
    }
}