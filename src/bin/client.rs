use chatbot::client;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => {
            //TODO validate arg?
            client::run(&args[1]);
        }
        _ => println!("Please provide exactly one argument: the address of the server"),
    }
}
