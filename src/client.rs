//use std::io::prelude::*;
//use std::net::TcpStream;
//use std::net::Shutdown;
//use std::process;

use std::fmt;
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

    //Get terminal dimensions
    let (width, height) = terminal_size().unwrap();

    //Get raw terminal
    let mut stdout = stdout().into_raw_mode().unwrap();

    //Get stdin
    let mut stdin = termion::async_stdin().keys();

    let mut msg_buf = Buf::init();
    let mut msg_log: Vec<String> = Vec::new();

    loop {
        //Draw the screen
        redraw(&mut stdout, height, width, &msg_buf, &msg_log);

        //Handle input
        if let Some(action) = handle_input(&mut stdin, &mut msg_buf) {
            match action {
                Action::Quit => break,
                Action::Clear => {
                    msg_log.push(String::from(format!("{}", msg_buf)));
                    msg_buf.clear();
                }
            }
        }

        thread::sleep(Duration::from_millis(10));
    }
}

fn redraw(
    stdout: &mut RawTerminal<Stdout>,
    height: u16,
    width: u16,
    msg_buf: &Buf,
    msg_log: &Vec<String>,
) {
    //Draw divider
    write!(
        stdout,
        "{}{}{}",
        cursor::Goto(0, height - 2),
        clear::CurrentLine,
        "=".repeat(width.into()),
    )
    .unwrap();
    write!(
        stdout,
        "{}{}Type your message here:",
        cursor::Goto(0, height - 1),
        clear::CurrentLine,
    )
    .unwrap();

    //Draw message buffer
    write!(
        stdout,
        "{}{}{}",
        cursor::Goto(0, height),
        clear::CurrentLine,
        msg_buf,
    )
    .unwrap();

    //Draw log
    for i in 0..height - 3 {
        if usize::from(i) >= msg_log.len() {
            break;
        }

        write!(
            stdout,
            "{}{}{}",
            cursor::Goto(0, height - (i + 3)),
            clear::CurrentLine,
            msg_log[msg_log.len() - (usize::from(i) + 1)],
        )
        .unwrap();
    }

    stdout.flush().unwrap();
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
