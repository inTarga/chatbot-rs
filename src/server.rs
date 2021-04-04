use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

pub fn serve() {
    let listener = TcpListener::bind("localhost:7878").expect("Failed to bind port");

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(mut stream) => handle_connection(&mut stream),
            Err(error) => eprintln!("failed to resolve stream: {}", error),
        }
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
