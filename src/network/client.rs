use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug)]
pub struct Client {
    pub stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Client {
        Client { stream }
    }

    pub fn read(&mut self) -> Option<String> {
        let mut message_bytes = [0; 512];
        self.stream.set_read_timeout(Some(Duration::from_millis(10))).unwrap();
        let num_bytes_read = self.stream.read(&mut message_bytes).unwrap_or_else(|_| 0);
        if num_bytes_read > 0 {
            let mut message = String::from(std::str::from_utf8(&message_bytes).unwrap());
            message.truncate(num_bytes_read);
            return Some(message);
        }
        None
    }

    pub fn write_stream(&mut self, message: &str) {
        if message.len() == 0 {
            return;
        }
        let stream = &mut &self.stream;
        let result = stream.write(message.as_bytes());

        match result {
            Ok(_) => {
                eprintln!("Message Sent");
            },
            _ => eprintln!("couldnt connect to ip:port")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::thread;
    use super::*;

    fn setup_test_server() -> (TcpListener, String) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap(); // Bind to any available port
        let addr = listener.local_addr().unwrap().to_string(); // Get the assigned port
        (listener, addr)
    }

    fn setup_client(addr: &str) -> Client {
        let tcp_stream = TcpStream::connect(addr).unwrap();
        Client::new(tcp_stream)
    }

    #[test]
    fn test_client_can_connect() {
        let (listener, addr) = setup_test_server();

        thread::spawn(move || {
            let _ = setup_client(&addr);
        });

        // Accept connection
        let (stream, _) = listener.accept().unwrap();
        assert!(stream.peer_addr().is_ok(), "Client should connect successfully");
    }

    #[test]
    fn test_client_can_send_message() {
        let (listener, addr) = setup_test_server();

        thread::spawn(move || {
            let mut client = setup_client(&addr);
            client.write_stream("Hello, Server!");
        });

        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0; 512];
        let bytes_read = stream.read(&mut buffer).unwrap();
        let received_message = String::from_utf8_lossy(&buffer[..bytes_read]);

        assert_eq!(received_message, "Hello, Server!", "Client should send a message");
    }

    #[test]
    fn test_client_can_receive_message() {
        let (listener, addr) = setup_test_server();

        thread::spawn(move || {
            let mut client = setup_client(&addr);
            let received = client.read();
            assert_eq!(received, Some("Hello, Client!".to_string()), "Client should receive a message");
        });

        let (mut stream, _) = listener.accept().unwrap();
        stream.write_all(b"Hello, Client!").unwrap();
    }

}