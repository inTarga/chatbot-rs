use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn run_bot(
    rcv: mpsc::Receiver<String>,
    snd: mpsc::Sender<String>,
    bot: &dyn Fn(Vec<String>) -> String,
) {
    //things the bots can understand and respond to
    let known_actions: [&str; 4] = ["eat", "sleep", "code", "cycle"];
    loop {
        match rcv.try_recv() {
            Ok(msg) => {
                //find out what actions are mentioned
                let actions = parse_actions(msg, &known_actions);
                //send back what the bot thought of those actions
                snd.send(bot(actions)).unwrap_or(());
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                //TODO: log?
                break;
            }
            Err(mpsc::TryRecvError::Empty) => (), //if there's no msg, just move on
        };

        //TODO: randomise delay?
        thread::sleep(Duration::from_secs(1));
    }
}

fn parse_actions(msg: String, known_actions: &[&str]) -> Vec<String> {
    let mut actions = Vec::new();

    for action in known_actions {
        if msg.contains(action) {
            actions.push(action.to_string());
        }
    }

    actions
}

pub fn gen_text(actions: Vec<String>) -> String {
    if actions.len() > 0 {
        format!("Hmm... I don't want to {}", actions[0])
    } else {
        "What are you on about?".to_string()
    }
}
