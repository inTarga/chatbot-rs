use std::io::prelude::*;
use std::net::TcpStream;
//use std::net::Shutdown;
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

    //Draw log
    //iterate from line above message buffer to top of term
    //TODO: Better breaking
    let mut i = 0;
    while i < height - 3 {
        //break if we run out of messages
        if usize::from(i / 2) >= msg_log.len() {
            break;
        }

        let msg_index = msg_log.len() - (usize::from(i / 2) + 1);
        //TODO: break these writes out into a function?
        write!(
            stdout,
            "{}{}{}{}{}",
            cursor::Goto(0, height - (i + 4)),
            clear::CurrentLine,
            color::Fg(color::Red),
            style::Bold,
            msg_log[msg_index].author,
        )?;
        write!(
            stdout,
            "{}{}{}{}{}",
            cursor::Goto(0, height - (i + 3)),
            clear::CurrentLine,
            color::Fg(color::White),
            style::Reset,
            msg_log[msg_index].body,
        )?;

        i += 2;
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
