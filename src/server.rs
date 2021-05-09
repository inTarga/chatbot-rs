use crate::bots;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn serve() {
    let listener = TcpListener::bind("localhost:7878").expect("Failed to bind port");

    for stream_result in listener.incoming() {
        thread::spawn(|| match stream_result {
            Ok(mut stream) => handle_connection(&mut stream),
            Err(error) => eprintln!("failed to resolve stream: {}", error),
        });
    }
}

fn handle_connection(stream: &mut TcpStream) {
    stream
        .set_read_timeout(Some(Duration::from_millis(10)))
        .expect("Failed to set read timeout on TCP stream");
    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream"));

    //Create bots
    let (snd_out, rcv_out): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
    let (snd_in, rcv_in): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
    thread::spawn(move || bots::run_bot(rcv_in, snd_out, &bots::gen_text));

    loop {
        //TODO: prevent constant reallocation?
        let mut buffer = String::with_capacity(1024);
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                //TODO: log?
                break;
            }
            Ok(_) => {
                buffer.pop();
                snd_in.send(buffer).unwrap();
            }
            _ => (),
        };

        match rcv_out.try_recv() {
            Ok(msg) => match forward_reply(stream, msg, String::from("Alice :")) {
                Ok(_) => (),
                Err(_) => {
                    //TODO: log?
                    break;
                }
            },
            Err(_) => (),
        }

        thread::sleep(Duration::from_millis(10));
    }
}

//Forward a reply from a bot to the client
fn forward_reply(stream: &mut TcpStream, msg: String, author: String) -> std::io::Result<()> {
    stream.write(author.as_bytes())?;
    stream.write(msg.as_bytes())?;
    stream.write(b"\n")?;
    Ok(())
}
