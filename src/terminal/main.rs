#![deny(warnings)]
#![feature(const_fn)]

extern crate orbclient;

use orbclient::event;

use std::{env, str, thread};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write, self};
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use console::Console;

mod console;

fn main() {
    let shell = env::args().nth(1).unwrap_or("sh".to_string());

    let master = File::create("pty:").unwrap();
    let tty_path = master.path().unwrap();
    let master_fd = master.into_raw_fd();

    let slave_stdin = File::open(&tty_path).unwrap();
    let slave_stdout = File::open(&tty_path).unwrap();
    let slave_stderr = File::open(&tty_path).unwrap();

    let width = 640;
    let height = 480;

    env::set_var("COLUMNS", format!("{}", width/8));
    env::set_var("LINES", format!("{}", height/16));
    env::set_var("TTY", format!("{}", tty_path.display()));

    match unsafe {
        Command::new(&shell)
            .stdin(Stdio::from_raw_fd(slave_stdin.into_raw_fd()))
            .stdout(Stdio::from_raw_fd(slave_stdout.into_raw_fd()))
            .stderr(Stdio::from_raw_fd(slave_stderr.into_raw_fd()))
            .spawn()
    } {
        Ok(_process) => {
            let output_mutex = Arc::new(Mutex::new(Some(Vec::new())));
            let mut master_stdin = unsafe { File::from_raw_fd(master_fd) };

            {
                let stdout_output_mutex = output_mutex.clone();
                thread::spawn(move || {
                    let mut master_stdout = unsafe { File::from_raw_fd(master_fd) };
                    let term_stderr = io::stderr();
                    let mut term_stderr = term_stderr.lock();
                    'stdout: loop {
                        let mut buf = [0; 4096];
                        match master_stdout.read(&mut buf) {
                            Ok(0) => break 'stdout,
                            Ok(count) => match stdout_output_mutex.lock() {
                                Ok(mut stdout_output_option) => match *stdout_output_option {
                                    Some(ref mut stdout_output) => stdout_output.push((buf[0], Vec::from(&buf[1..count]))),
                                    None => break 'stdout
                                },
                                Err(_) => {
                                    let _ = term_stderr.write(b"failed to lock stdout output mutex\n");
                                    break 'stdout;
                                }
                            },
                            Err(err) => {
                                let _ = term_stderr.write(b"failed to read stdout: ");
                                let _ = term_stderr.write(err.description().as_bytes());
                                let _ = term_stderr.write(b"\n");
                                break 'stdout;
                            }
                        }
                    }
                    stdout_output_mutex.lock().unwrap().take();
                });
            }

            let mut console = Console::new(width, height);
            'events: loop {
                match output_mutex.lock() {
                    Ok(mut output_option) => match *output_option {
                        Some(ref mut output) => for packet in output.drain(..) {
                            if packet.0 == 1 {
                                console.inner.redraw = true;
                            }
                            console.write(&packet.1)
                        },
                        None => break 'events
                    },
                    Err(_) => {
                        let term_stderr = io::stderr();
                        let mut term_stderr = term_stderr.lock();
                        let _ = term_stderr.write(b"failed to lock stdout mutex\n");
                        break 'events;
                    }
                }

                for event in console.window.events() {
                    if event.code == event::EVENT_QUIT {
                        break 'events;
                    }

                    if let Some(line) = console.event(event) {
                        if let Err(err) = master_stdin.write(&line.as_bytes()) {
                            let term_stderr = io::stderr();
                            let mut term_stderr = term_stderr.lock();

                            let _ = term_stderr.write(b"failed to write stdin: ");
                            let _ = term_stderr.write(err.description().as_bytes());
                            let _ = term_stderr.write(b"\n");
                            break 'events;
                        }
                    }
                }

                thread::sleep(Duration::new(0, 1000000));
            }

            output_mutex.lock().unwrap().take();
        },
        Err(err) => {
            let term_stderr = io::stderr();
            let mut term_stderr = term_stderr.lock();
            let _ = term_stderr.write(b"failed to execute '");
            let _ = term_stderr.write(shell.as_bytes());
            let _ = term_stderr.write(b"': ");
            let _ = term_stderr.write(err.description().as_bytes());
            let _ = term_stderr.write(b"\n");
        }
    }
}
