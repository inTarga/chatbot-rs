use rand::seq::SliceRandom;
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
                let actions = parse_actions(msg.to_lowercase(), &known_actions);
                //send back what the bot thought of those actions
                let reply = bot(actions);
                if reply.len() > 0 {
                    snd.send(reply).unwrap_or(());
                }
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

pub fn alice(actions: Vec<String>) -> String {
    let prefix = "Alice :";
    if actions.len() > 0 {
        let response = ["Hmm... I don't want to ", "I don't really feel up for a "]
            .choose(&mut rand::thread_rng())
            .unwrap();
        format!("{}{}{}", prefix, response, actions[0])
    } else {
        let response = ["What are you on about?", "I literally can't even..."]
            .choose(&mut rand::thread_rng())
            .unwrap();
        format!("{}{}", prefix, response)
    }
}

pub fn beate(actions: Vec<String>) -> String {
    "Beate :placeholder".to_string()
}
