use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpListener;
use std::net::TcpStream;

//mod server;

pub fn serve() {
    let listener = TcpListener::bind("localhost:7878").unwrap();

    for stream in listener.incoming() {
        handle_connection(stream.unwrap());
    }
}

fn handle_connection(stream: TcpStream) {
    let mut reader = BufReader::new(stream);
    let mut header = String::with_capacity(1024);

    reader.read_line(&mut header).unwrap();
    header.pop();

    println!("Header: {}", header);

    match header.as_str() {
        "WRITE" => println!("Write!"),
        _ => println!("Header not recognised...")
    }
}
