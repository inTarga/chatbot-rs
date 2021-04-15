use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn run_bot(rcv: mpsc::Receiver<String>, snd: mpsc::Sender<String>) {
    loop {
        match rcv.try_recv() {
            Ok(msg) => snd.send(gen_text(msg)).unwrap_or(()),
            Err(mpsc::TryRecvError::Disconnected) => {
                //TODO: log?
                break;
            }
            Err(mpsc::TryRecvError::Empty) => (),
        };

        thread::sleep(Duration::from_secs(1));
    }
}

fn gen_text(in_msg: String) -> String {
    format!("Hmm... I don't know about \"{}\"", in_msg)
}
