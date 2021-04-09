use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn run_bot(rcv: mpsc::Receiver<String>, snd: mpsc::Sender<String>) {
    loop {
        match rcv.try_recv() {
            Ok(msg) => snd.send(gen_text(msg)).unwrap(),
            _ => (),
        };

        thread::sleep(Duration::from_millis(10));
    }
}

fn gen_text(in_msg: String) -> String {
    format!("Hmm... I don't know about \"{}\"", in_msg)
}
