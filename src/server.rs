use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

pub fn serve() {
    let listener = TcpListener::bind("localhost:7878").unwrap();

    for stream in listener.incoming() {
        handle_connection(&mut stream.unwrap());
    }
}

fn handle_connection(stream: &mut TcpStream) {
    //TODO: prevent leak/panic
    loop {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();

        println!("received message: {}", String::from_utf8_lossy(&buffer));
        stream.write(b"received message: ").unwrap();
    }
}
