use chrono::Utc;
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::prelude::*;
use std::io::{self, stdin, stdout, BufReader, Stdout, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use termion;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style, terminal_size};

pub fn run() {
    //Clear screen
    println!("{}", clear::All);

    //Connect to the server
    let stream = TcpStream::connect("localhost:7878").expect("Failed to connect to server");
    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream"));

    //Get raw terminal
    let stdout = stdout()
        .into_raw_mode()
        .expect("Failed to put terminal into raw mode");

    //Get stdin
    let stdin = stdin();

    //Initialise state
    let msg_buf: Vec<char> = Vec::new();
    let msg_log: Vec<Msg> = Vec::new();
    let mut colourmap: HashMap<String, String> = HashMap::new();
    colourmap.insert("You".to_string(), color::Red.fg_str().to_string());

    //Construct IO threads
    let (io_snd_net, io_rcv) = mpsc::channel::<IOEvent>();
    let io_snd_key = io_snd_net.clone();

    thread::spawn(move || {
        for c in stdin.keys() {
            io_snd_key.send(IOEvent::Key(c)).unwrap();
        }
    });

    thread::spawn(move || loop {
        let mut buffer = String::with_capacity(1024);

        io_snd_net
            .send(match BufRead::read_line(&mut reader, &mut buffer) {
                Ok(_) => IOEvent::Msg(Ok(buffer)),
                Err(err) => IOEvent::Msg(Err(err)),
            })
            .unwrap()
    });

    //Handle IO events as they are received from the threads
    handle_events(io_rcv, stdout, stream, msg_buf, msg_log, colourmap);
}

fn handle_events(
    events: mpsc::Receiver<IOEvent>,
    mut stdout: RawTerminal<Stdout>,
    mut stream: TcpStream,
    mut msg_buf: Vec<char>,
    mut msg_log: Vec<Msg>,
    mut colourmap: HashMap<String, String>,
) {
    loop {
        //Get terminal dimensions
        let (width, height) = terminal_size().expect("Failed to get terminal size");

        //Draw the screen
        redraw(&mut stdout, height, width, &msg_buf, &msg_log, &colourmap)
            .expect("Failed to write");

        match events.recv() {
            //Handle input
            Ok(IOEvent::Key(c)) => {
                //TODO: remove unwrap?
                if let Some(action) = process_key(c.unwrap(), &mut msg_buf) {
                    match action {
                        Action::Quit => break,
                        Action::Clear => send_message(&mut stream, &mut msg_buf, &mut msg_log)
                            .expect("Failed to send message"),
                    }
                }
            }
            //Handle new message
            Ok(IOEvent::Msg(msg)) => {
                process_msg(msg.unwrap(), &mut msg_log, &mut colourmap);
            }
            _ => (), //TODO: handle error cases?
        }
    }

    write!(stdout, "{}{}", cursor::Goto(1, 1), clear::All).expect("Failed to clear the terminal");
}

fn process_key(key: Key, msg_buf: &mut Vec<char>) -> Option<Action> {
    //TODO: handle signals
    match key {
        Key::Ctrl('c') => return Some(Action::Quit),
        Key::Char('\n') => return Some(Action::Clear),
        Key::Char(c) => msg_buf.push(c),
        Key::Backspace => {
            msg_buf.pop();
        }
        _ => (),
    }
    return None;
}

fn process_msg(raw_msg: String, msg_log: &mut Vec<Msg>, colourmap: &mut HashMap<String, String>) {
    //Find separator between author and body
    match raw_msg.find(":") {
        Some(i) => {
            let (author, body) = raw_msg.split_at(i + 1);
            let author_string = String::from(author.trim_end_matches(":"));
            colour_author(author_string.clone(), colourmap);
            msg_log.push(Msg {
                author: author_string,
                time: timestamp(),
                body: String::from(body),
            });
        }
        None => (), //TODO: log?
    };
}

fn redraw(
    stdout: &mut RawTerminal<Stdout>,
    height: u16,
    width: u16,
    msg_buf: &Vec<char>,
    msg_log: &Vec<Msg>,
    colourmap: &HashMap<String, String>,
) -> std::io::Result<()> {
    //TODO: return early if term is too small?

    let mut lines: Vec<String> = Vec::new(); //lines to be displayed

    //Prepare log
    //loop over messages in log, reversed to show newest messages first
    for msg in msg_log.iter() {
        //first line is the author line
        lines.push(String::from(format!(
            "{}{} {}{}{}{}",
            style::Bold,
            msg.time,
            colourmap //get author's colour, default to Fg
                .get(&msg.author)
                .unwrap_or(&color::White.fg_str().to_string()),
            msg.author,
            color::Fg(color::White), //remove formatting before moving on
            style::Reset,
        )));

        //split the message body into lines according to terminal width
        split_and_push(msg.body.clone(), &mut lines, usize::from(width));
    }

    //Prepare divider
    lines.push(format!("{}{}", "─".repeat(width.into()), style::Reset));
    lines.push(format!("{}{}", style::Bold, "Type your message here:"));

    //Prepare message buffer
    split_and_push(msg_buf.iter().collect(), &mut lines, usize::from(width));

    let mut line_num = height; //line number we're drawing at, start from bottom

    //iterate over the message, and write each line to the terminal,
    //reversed because we write from the bottom up
    for line in lines.iter().rev() {
        write!(
            stdout,
            "{}{}{}",
            cursor::Goto(0, line_num),
            clear::CurrentLine,
            line,
        )?;

        //if we've reached the top of the terminal, to stop writing any messages
        if line_num == 1 {
            break;
        }
        line_num -= 1;
    }

    //clear the rest of the screen to prevent zombie writes
    //TODO: perhaps this can be merged into the above for loop?
    if line_num > 1 {
        for y in 1..=line_num {
            write!(stdout, "{}{}", cursor::Goto(0, y), clear::CurrentLine)?;
        }
    }

    //fix cursor position
    let cursor_x = lines[lines.len() - 1].len() + 1; // x position of cursor should be end of the last line, +1 converts to 1-based indexing for termion
    write!(
        stdout,
        "{}",
        cursor::Goto(cursor_x.try_into().unwrap_or_default(), height)
    )?;

    stdout.flush()
}

fn colour_author(author: String, colourmap: &mut HashMap<String, String>) {
    match colourmap.get(&author) {
        Some(_) => (),
        None => {
            let colours = [
                color::Red.fg_str(),
                color::Green.fg_str(),
                color::Yellow.fg_str(),
                color::Blue.fg_str(),
                color::Magenta.fg_str(),
                color::Cyan.fg_str(),
            ];
            let i = colourmap.len() % colours.len();
            let colour = colours[i];
            colourmap.insert(author, colour.to_string());
        }
    }
}

fn send_message(
    stream: &mut TcpStream,
    msg_buf: &mut Vec<char>,
    msg_log: &mut Vec<Msg>,
) -> std::io::Result<()> {
    let msg = Msg {
        author: String::from("You"),
        time: timestamp(),
        body: msg_buf.iter().collect(),
    };

    stream.write(msg.body.as_bytes())?;
    stream.write(b"\n")?;
    msg_log.push(msg);

    msg_buf.clear();
    Ok(())
}

//TODO: do something clever with iterators?
fn split_and_push(src: String, dst: &mut Vec<String>, width: usize) {
    let mut rest = src;
    while rest.len() > width {
        let split = rest.split_at(width);
        dst.push(String::from(split.0));
        rest = String::from(split.1);
    }
    dst.push(rest);
}

fn timestamp() -> String {
    format!("{}", Utc::now().format("%T"))
}

enum IOEvent {
    Key(Result<Key, io::Error>),
    Msg(Result<String, io::Error>),
}

enum Action {
    Quit,
    Clear,
}

struct Msg {
    author: String,
    time: String,
    body: String,
}
