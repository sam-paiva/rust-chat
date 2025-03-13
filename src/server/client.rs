use std::{io, thread};
use std::io::{BufRead, BufReader, Read, Write};
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
                eprintln!("Written successfully");
            },
            _ => eprintln!("couldnt connect to ip:port")
        }
    }
}