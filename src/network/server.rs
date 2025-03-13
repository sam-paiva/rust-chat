use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use crate::network::client::Client;

pub enum ServerCommand {
    Connect(String),
    SendMessage(String),
}

pub struct Server {
    listener: Arc<TcpListener>,
    clients: Arc<Mutex<Vec<Client>>>,
}

impl Server {
    pub fn new(ip: &str) -> Server {
        let listener_result = TcpListener::bind(ip);

        let listener = match listener_result {
            Ok(listener) => {
                listener
            }
            Err(err) => {
                panic!("Can't listen on this IP address: {}. Err: {}", ip, err);
            }
        };

        println!("Listening on {}", ip);
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
        if clients.is_empty() {
            println!("No clients connected yet!");
            return;
        }

        for client in clients.iter_mut() {
            client.write_stream(message.as_str());
        }
    }

    fn connect(mut clients: MutexGuard<Vec<Client>>, ip: &str) {
        let stream_result = TcpStream::connect(ip);

        match stream_result {
            Ok(stream) => {
                let client = Client::new(stream.try_clone().unwrap());
                clients.push(client);
                println!("🆗 Connected successfully!");
            },
            Err(err) => {
                println!("Can't connect to the ip: {}. Err: {}", ip, err);
            }
        }
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
                            println!("\n✅ New Message: {}", data);
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
                    Ok(stream) => {
                        println!("🆗New connection: {}", stream.try_clone().unwrap().peer_addr().unwrap());

                        let client = Client::new(stream.try_clone().unwrap());
                        temp_clients.lock().unwrap().push(client);
                    }
                    _ => {}
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;
    use super::*;
    fn setup_test_server() -> (Server, mpsc::Sender<ServerCommand>) {
        let server = Server::new("127.0.0.1:0");
        let (tx, rx) = mpsc::channel();
        server.handle_commands(rx);

        (server, tx)
    }

    #[test]
    fn test_initialization() {
        let (server, _) = setup_test_server();
        assert!(server.clients.lock().unwrap().is_empty(), "Server should start with no clients");
    }

    #[test]
    fn test_client_connection() {
        let (server, tx) = setup_test_server();
        let addr = server.listener.local_addr().unwrap().to_string();

        thread::spawn(move || {
            tx.send(ServerCommand::Connect(addr)).unwrap();
        });

        thread::sleep(Duration::from_secs(1)); // Allow time for the thread to execute

        let clients = server.clients.lock().unwrap();
        assert_eq!(clients.len(), 1, "Client should be connected successfully");
    }

    #[test]
    fn test_send_message_to_clients() {
        let (server, tx) = setup_test_server();
        let address = server.listener.local_addr().unwrap().to_string();

        let arc_server = Arc::new(server);
        let server_clone = arc_server.clone();
        thread::spawn(move || {
            let stream = TcpStream::connect(&address).unwrap();
            let client = Client::new(stream);
            server_clone.clients.lock().unwrap().push(client);

            tx.send(ServerCommand::SendMessage("Hello".to_string())).unwrap();
        });

        thread::sleep(Duration::from_secs(1));
        let server_clone = arc_server.clone();
        let arc_server = Arc::new(server_clone);
        let clients = arc_server.clients.lock().unwrap();
        assert!(!clients.is_empty(), "There should be at least one client connected");
    }
}