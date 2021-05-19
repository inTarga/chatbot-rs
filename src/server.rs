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

    let mut bot_senders = Vec::new();
    let mut bot_receivers = Vec::new();
    //Create bots
    for bot in [
        bots::alice,
        bots::beate,
        bots::cara,
        bots::divya,
        bots::eudora,
    ]
    .iter()
    {
        let (snd_out, rcv_out): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();
        let (snd_in, rcv_in): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();

        thread::spawn(move || bots::run_bot(rcv_in, snd_out, &bot));

        bot_senders.push(snd_in);
        bot_receivers.push(rcv_out);
    }

    loop {
        //TODO: prevent constant reallocation?
        let mut buffer = String::with_capacity(1024);
        // try to read tcp stream from client
        match reader.read_line(&mut buffer) {
            Ok(0) => {
                // EOF, client disconnected, so should we
                //TODO: log?
                break;
            }
            Ok(_) => {
                // got a message, forward to bots
                buffer.pop();
                for sender in &bot_senders {
                    sender.send(buffer.clone()).unwrap(); // is clone really the best way of proceeding here?
                }
            }
            _ => (), // ignore timeout and move on
        };

        for receiver in &bot_receivers {
            match receiver.try_recv() {
                Ok(msg) => match forward_reply(stream, msg) {
                    Ok(_) => (),
                    Err(_) => {
                        //TODO: log?
                        break; // should this break outer?
                    }
                },
                Err(_) => (),
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}

//Forward a reply from a bot to the client
fn forward_reply(stream: &mut TcpStream, msg: String) -> std::io::Result<()> {
    stream.write(msg.as_bytes())?;
    stream.write(b"\n")?;
    Ok(())
}
