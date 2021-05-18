use chrono::Utc;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::io::prelude::*;
use std::io::{stdout, BufReader, Stdout, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use termion;
use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style, terminal_size};

pub fn run() {
    //Clear screen
    println!("{}", clear::All);

    //Connect to the server
    let mut stream = TcpStream::connect("localhost:7878").expect("Failed to connect to server");
    stream
        .set_read_timeout(Some(Duration::from_millis(10)))
        .expect("Failed to set read timeout");
    let mut reader = BufReader::new(stream.try_clone().expect("Failed to clone stream"));

    //Get raw terminal
    let mut stdout = stdout()
        .into_raw_mode()
        .expect("Failed to put terminal into raw mode");

    //Get stdin
    let mut stdin = termion::async_stdin().keys();

    let mut msg_buf = Buf::init();
    let mut msg_log: Vec<Msg> = Vec::new();
    let mut colourmap: HashMap<String, String> = HashMap::new();
    colourmap.insert("You".to_string(), color::Red.fg_str().to_string());

    loop {
        //Get terminal dimensions
        let (width, height) = terminal_size().expect("Failed to get terminal size");

        //Poll the server
        poll_server(&mut reader, &mut msg_log, &mut colourmap);

        //Draw the screen
        redraw(&mut stdout, height, width, &msg_buf, &msg_log, &colourmap)
            .expect("Failed to write");

        //Handle input
        if let Some(action) = handle_input(&mut stdin, &mut msg_buf) {
            match action {
                Action::Quit => break,
                Action::Clear => send_message(&mut stream, &mut msg_buf, &mut msg_log)
                    .expect("Failed to send message"),
            }
        }

        thread::sleep(Duration::from_millis(10));
    }

    write!(stdout, "{}{}", cursor::Goto(1, 1), clear::All).expect("Failed to clear the terminal");
}

fn poll_server(
    reader: &mut BufReader<TcpStream>,
    msg_log: &mut Vec<Msg>,
    colourmap: &mut HashMap<String, String>,
) {
    let mut buffer = String::with_capacity(1024);

    match BufRead::read_line(reader, &mut buffer) {
        Ok(_) => match buffer.find(":") {
            Some(i) => {
                let (author, body) = buffer.split_at(i + 1);
                let author_string = String::from(author.trim_end_matches(":"));
                colour_author(author_string.clone(), colourmap);
                msg_log.push(Msg {
                    author: author_string,
                    time: timestamp(),
                    body: String::from(body),
                });
            }
            None => (), //TODO: log?
        },
        _ => (), //TODO: log?
    }
}

fn redraw(
    stdout: &mut RawTerminal<Stdout>,
    height: u16,
    width: u16,
    msg_buf: &Buf,
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
    lines.push(format!("{}{}", "=".repeat(width.into()), style::Reset));
    lines.push(format!("{}{}", style::Bold, "Type your message here:"));

    //Prepare message buffer
    split_and_push(format!("{}", msg_buf), &mut lines, usize::from(width));

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

    //fix cursor position
    let cursor_x = lines[lines.len() - 1].len() + 1; // x position of cursor should be end of the last line, +1 converts to 1-based indexing for termion
    write!(
        stdout,
        "{}",
        cursor::Goto(cursor_x.try_into().unwrap_or_default(), height)
    )?;

    stdout.flush()
}

fn handle_input(keys: &mut Keys<termion::AsyncReader>, msg_buf: &mut Buf) -> Option<Action> {
    //TODO: handle more keys at once?
    //TODO: handle signals
    if let Some(Ok(key)) = keys.next() {
        match key {
            Key::Ctrl('c') => return Some(Action::Quit),
            Key::Char('\n') => return Some(Action::Clear),
            Key::Char(c) => msg_buf.insert(c),
            Key::Backspace => msg_buf.back(),
            _ => (),
        }
    }
    return None;
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
    msg_buf: &mut Buf,
    msg_log: &mut Vec<Msg>,
) -> std::io::Result<()> {
    let msg = Msg {
        author: String::from("You"),
        time: timestamp(),
        body: String::from(format!("{}", msg_buf)),
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

enum Action {
    Quit,
    Clear,
}

struct Msg {
    author: String,
    time: String,
    body: String,
}

struct Buf {
    buffer: [char; 1024],
    head: usize,
}

impl Buf {
    fn init() -> Buf {
        Buf {
            buffer: [' '; 1024],
            head: 0,
        }
    }

    //TODO: handle overflow
    fn insert(&mut self, c: char) {
        self.buffer[self.head] = c;
        self.head += 1;
    }

    //TODO: handle underflow
    fn back(&mut self) {
        self.head -= 1;
        self.buffer[self.head] = ' ';
    }

    fn clear(&mut self) {
        for i in 0..(self.head) {
            self.buffer[i] = ' ';
        }
        self.head = 0;
    }
}

impl fmt::Display for Buf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let out = &self.buffer[0..self.head];
        write!(f, "{}", out.into_iter().collect::<String>())
    }
}
