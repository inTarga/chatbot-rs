//use std::io::prelude::*;
//use std::net::TcpStream;
//use std::net::Shutdown;
//use std::process;

use std::io::{Write, stdout, Stdout};
use termion::event::Key;
use termion::input::Keys;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion;
use termion::cursor;
use termion::clear;
use termion::terminal_size;
use std::thread;
use std::time::Duration;

pub fn run() {
    //Clear screen
    println!("{}", clear::All);

    //Get terminal dimensions
    let (width, height) = terminal_size().unwrap();

    //Get raw terminal
    let mut stdout = stdout().into_raw_mode().unwrap();

    //Get stdin
    let mut stdin = termion::async_stdin().keys();

    loop {
        redraw(&mut stdout, height, width);
        let quit = handle_input(&mut stdin);

        if quit {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn redraw(stdout: &mut RawTerminal<Stdout>, height: u16, width: u16) {
    //Draw divider
    writeln!(stdout, "{}{}", cursor::Goto(0, height-2), "=".repeat(width.into())).unwrap();
    writeln!(stdout, "{}Type your message here:", cursor::Goto(0, height-1)).unwrap();

    for i in 0..height-3 {
        writeln!(stdout, "{}message{}", cursor::Goto(0, height-(i+3)), i.to_string()).unwrap();
    }
}

fn handle_input(keys: &mut Keys<termion::AsyncReader>) -> bool {
    let input = keys.next();
    if let Some(Ok(key)) = input {
        match key {
            //Key::Char('q') => process::exit(0),
            Key::Char('q') => return true,
            _ => (),
        }
    }
    return false;
}

