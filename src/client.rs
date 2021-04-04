use std::io::prelude::*;
use std::net::TcpStream;
//use std::net::Shutdown;
//use std::process;

use std::fmt;
//use std::io;
use std::io::{stdout, Stdout, Write};
use std::thread;
use std::time::Duration;
use termion;
use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor, terminal_size};

pub fn run() {
    //Clear screen
    println!("{}", clear::All);

    //Connect to the server
    let mut stream = TcpStream::connect("localhost:7878").expect("Failed to connect to server");
    stream
        .set_read_timeout(Some(Duration::from_millis(10)))
        .expect("Failed to set read timeout");

    //Get terminal dimensions
    let (width, height) = terminal_size().expect("Failed to get terminal size");

    //Get raw terminal
    let mut stdout = stdout()
        .into_raw_mode()
        .expect("Failed to put terminal into raw mode");

    //Get stdin
    let mut stdin = termion::async_stdin().keys();

    let mut msg_buf = Buf::init();
    let mut msg_log: Vec<String> = Vec::new();

    loop {
        //Poll the server
        poll(&mut stream, &mut msg_log);

        //Draw the screen
        redraw(&mut stdout, height, width, &msg_buf, &msg_log).expect("Failed to write");

        //Handle input
        if let Some(action) = handle_input(&mut stdin, &mut msg_buf) {
            match action {
                Action::Quit => break,
                Action::Clear => send_message(&mut stream, &mut msg_buf, &mut msg_log).expect("Failed to send message"),
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}

fn poll(stream: &mut TcpStream, msg_log: &mut Vec<String>) {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(_) => msg_log
            .push(String::from_utf8(buffer.to_vec()).expect("Failed to parse message from server")),
        _ => (),
    }
}

fn redraw(
    stdout: &mut RawTerminal<Stdout>,
    height: u16,
    width: u16,
    msg_buf: &Buf,
    msg_log: &Vec<String>,
) -> std::io::Result<()> {
    //Draw divider
    write!(
        stdout,
        "{}{}{}",
        cursor::Goto(0, height - 2),
        clear::CurrentLine,
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
        "{}{}{}",
        cursor::Goto(0, height),
        clear::CurrentLine,
        msg_buf,
    )?;

    //Draw log
    //iterate from line above message buffer to top of term
    for i in 0..height - 3 {
        //break if we run out of messages
        if usize::from(i) >= msg_log.len() {
            break;
        }

        write!(
            stdout,
            "{}{}{}",
            cursor::Goto(0, height - (i + 3)),
            clear::CurrentLine,
            msg_log[msg_log.len() - (usize::from(i) + 1)],
        )?;
    }

    stdout.flush()
}

fn handle_input(keys: &mut Keys<termion::AsyncReader>, msg_buf: &mut Buf) -> Option<Action> {
    //TODO: handle more keys at once?
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
    msg_log: &mut Vec<String>,
) -> std::io::Result<()> {
    let msg = String::from(format!("{}", msg_buf));

    stream.write(msg.as_bytes())?;
    stream.write(b"\n")?;
    msg_log.push(msg);

    msg_buf.clear();
    Ok(())
}

enum Action {
    Quit,
    Clear,
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
