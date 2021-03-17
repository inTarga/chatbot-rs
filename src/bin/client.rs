use std::io::prelude::*;
use std::net::TcpStream;

fn main() {
    let mut stream = TcpStream::connect("localhost:7878").unwrap();

    stream.write("Hello!".as_bytes()).unwrap();

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    println!("Response: {}", String::from_utf8_lossy(&buffer))
}
