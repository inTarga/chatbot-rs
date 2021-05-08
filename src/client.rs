use std::io::prelude::*;
//use std::net::Shutdown;
use std::net::TcpStream;
//use std::process;

use std::fmt;
//use std::io;
use std::io::{stdout, BufReader, Stdout, Write};
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

    //Get terminal dimensions
    //TODO: adjust dynamically
    let (width, height) = terminal_size().expect("Failed to get terminal size");

    //Get raw terminal
    let mut stdout = stdout()
        .into_raw_mode()
        .expect("Failed to put terminal into raw mode");

    //Get stdin
    let mut stdin = termion::async_stdin().keys();

    let mut msg_buf = Buf::init();
    let mut msg_log: Vec<Msg> = Vec::new();

    loop {
        //Poll the server
        poll_server(&mut reader, &mut msg_log);

        //Draw the screen
        redraw(&mut stdout, height, width, &msg_buf, &msg_log).expect("Failed to write");

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

fn poll_server(reader: &mut BufReader<TcpStream>, msg_log: &mut Vec<Msg>) {
    let mut buffer = String::with_capacity(1024);

    match BufRead::read_line(reader, &mut buffer) {
        Ok(_) => {
            let (author, body) = buffer.split_at(7);
            msg_log.push(Msg {
                author: String::from(author),
                body: String::from(body),
            });
        }
        _ => (),
    }
}

fn redraw(
    stdout: &mut RawTerminal<Stdout>,
    height: u16,
    width: u16,
    msg_buf: &Buf,
    msg_log: &Vec<Msg>,
) -> std::io::Result<()> {
    //Draw divider
    write!(
        stdout,
        "{}{}{}{}{}",
        cursor::Goto(0, height - 2),
        clear::CurrentLine,
        color::Fg(color::White),
        style::Bold,
        "=".repeat(width.into()),
    )?;
    write!(
        stdout,
        "{}{}Type your message here:",
        cursor::Goto(0, height - 1),
        clear::CurrentLine,
    )?;

    //Draw message buffer
    write!(
        stdout,
        "{}{}{}{}",
        cursor::Goto(0, height),
        clear::CurrentLine,
        style::Reset,
        msg_buf,
    )?;

    let mut line_num = height - 3; //line number we're drawing at, start from bottom

    //Draw log
    //loop over messages in log, reversed to show newest messages first
    'outer: for msg in msg_log.iter().rev() {
        let mut lines: Vec<String> = Vec::new(); //lines of the message to be displayed

        //first line is the author line,
        lines.push(String::from(format!(
            "{}{}{}{}{}",
            color::Fg(color::Red), //apply authorline formatting
            style::Bold,
            msg.author,
            color::Fg(color::White), //remove formatting before moving on
            style::Reset,
        )));

        //split the message body into lines according to terminal width
        split_and_push(msg.body.clone(), &mut lines, usize::from(width));

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
                break 'outer;
            }
            line_num -= 1;
        }
    }

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

fn send_message(
    stream: &mut TcpStream,
    msg_buf: &mut Buf,
    msg_log: &mut Vec<Msg>,
) -> std::io::Result<()> {
    let msg = Msg {
        author: String::from("You :"),
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

enum Action {
    Quit,
    Clear,
}

struct Msg {
    author: String,
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
