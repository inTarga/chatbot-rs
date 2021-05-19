use rand::seq::SliceRandom;
use rand::Rng;
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
        match rcv.recv() {
            Ok(msg) => {
                //find out what actions are mentioned
                let actions = parse_actions(msg.to_lowercase(), &known_actions);

                thread::sleep(Duration::from_millis(
                    rand::thread_rng().gen_range(100..2000),
                ));

                //send back what the bot thought of those actions
                let reply = bot(actions);
                if reply.len() > 0 {
                    snd.send(reply).unwrap_or(());
                }
            }
            Err(mpsc::RecvError) => {
                //TODO: log?
                break;
            }
        };
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
    let prefix = "Alice:";
    match actions.choose(&mut rand::thread_rng()) {
        Some(action) => {
            let response = ["Hmm... I don't want to ", "I don't really feel up for a "]
                .choose(&mut rand::thread_rng())
                .unwrap();
            format!("{}{}{}", prefix, response, action)
        }
        None => {
            let response = ["What are you on about?", "I literally can't even..."]
                .choose(&mut rand::thread_rng())
                .unwrap();
            format!("{}{}", prefix, response)
        }
    }
}

pub fn beate(actions: Vec<String>) -> String {
    if actions.len() == 0 {
        return "".to_string();
    }

    let action = actions.choose(&mut rand::thread_rng()).unwrap();
    let prefix = "Beate:";
    let alternative = ["harvesting", "slicing", "scalding"]
        .choose(&mut rand::thread_rng())
        .unwrap();
    format!(
        "{}A {} would be chill, But how about we do some {}!",
        prefix, action, alternative
    )
}

pub fn cara(actions: Vec<String>) -> String {
    if actions.len() > 0 {
        return "".to_string();
    }

    match rand::random() {
        true => {
            "Cara:It's chill, I didn't really feel like doing anything either. Let's just relax"
                .to_string()
        }
        false => {
            let alternatives: Vec<&&str> = ["sleep", "snooze", "doze", "nap", "slumber", "rest"]
                .choose_multiple(&mut rand::thread_rng(), 2)
                .collect();
            format!(
                "Cara:Hmmm, maybe we can {}... Ooh ooh I know, lets {}!",
                alternatives[0], alternatives[1]
            )
        }
    }
}

pub fn divya(actions: Vec<String>) -> String {
    match actions.len() {
        0 => format!("Divya:{}", "<3 ".repeat(30)),
        1 => format!("Divya:eat, sleep, {}, repeat", actions[0]),
        2 => format!("Divya:{}, sleep, {}, repeat", actions[0], actions[1]),
        _ => format!(
            "Divya:{}, {}, {}, repeat",
            actions[0], actions[1], actions[2]
        ),
    }
}

pub fn eudora(actions: Vec<String>) -> String {
    match actions.len() {
        0 => "Eudora:Couldn't even think of anything? Scrub.".to_string(),
        1 => "Eudora:One thing? Try harder.".to_string(),
        2 => "Eudora:Hmm... Maybe I underestimated you.".to_string(),
        _ => format!("Eudora:Wargh, {}?! I'm losing count!", actions.len()),
    }
}
