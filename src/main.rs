#![deny(clippy::all)]
#![deny(clippy::pedantic)]

#[macro_use]
mod helpers;
mod poll;
mod process;
mod signal;

use base64::encode;
use std::{
    env,
    io::{self, Read},
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(strum_macros::Display)]
pub enum PollType {
    #[strum(to_string = "pidexit")]
    PidExit,
    #[strum(to_string = "stdout")]
    Stdout,
    #[strum(to_string = "stderr")]
    Stderr,
}

/// Holds polling data
pub struct PollData {
    typ: PollType,
    pid: u32,
    data: String,
}

fn main() {
    let port_name = env::args()
        .nth(1)
        .expect("Port name argument (1) is required");

    // Open serial connection on /dev/hvc2, max baud rate
    let port = serialport::new(port_name, 115_200)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(mut port) => {
            // Hello message
            port.write_all("HELLO\0".as_bytes())
                .expect("Error writing to serial");

            let mut serial_buf: Vec<u8> = Vec::with_capacity(5000);
            let mut serial_char_buf: Vec<u8> = vec![0; 1];
            let poll_data: Arc<Mutex<Vec<PollData>>> = Arc::new(Mutex::new(vec![]));

            debug!("Receiving data on serial connection.");
            loop {
                loop {
                    if let Err(e) = port.read_exact(&mut serial_char_buf) {
                        match e.kind() {
                            io::ErrorKind::TimedOut | io::ErrorKind::UnexpectedEof => continue,
                            _ => panic!("IO error when reading character: {e}"),
                        }
                    }

                    match serial_char_buf.first() {
                        Some(i) => match i {
                            0x00 => break,
                            c => serial_buf.push(*c),
                        },
                        None => panic!("IO error when trying to read serial buffer"),
                    }
                }

                match process_cmds(&serial_buf, &poll_data) {
                    Ok(s) => {
                        port.write_all(format!("{}OK\0", &s).as_bytes())
                            .expect("Error writing to serial");
                    }
                    Err(e) => {
                        port.write_all(format!("{}\nERR\0", encode(&e)).as_bytes())
                            .expect("Error writing to serial");
                    }
                }
                serial_buf.clear();
                port.flush().expect("Couldn't flush serial on reading end");
            }
        }
        // If the port could not be opened, print an error and exit
        Err(e) => {
            eprintln!("Failed to open serial connection. Error: {e}");
            std::process::exit(1);
        }
    }
}

/// Process the serial buffer and parse commands therein
///
/// Commands:
/// - `run process_name args env_vars` - Run the specified process with the given arguments and environment variables
/// - `signal pid signal` - Send the given signal to the given pid
/// - `poll` - Poll for data, sends it all back
fn process_cmds(
    serial_buf: &[u8],
    poll_data: &Arc<Mutex<Vec<PollData>>>,
) -> Result<String, String> {
    // Tokenize and parse the command
    let cmd = String::from_utf8_lossy(serial_buf);
    let cmd_tokens: Vec<&str> = cmd.split(' ').collect();

    match cmd_tokens.as_slice() {
        ["run", process_name, args, env_vars] => {
            process::process(process_name, args, env_vars, poll_data)
        }
        ["signal", pid, signal] => signal::send(pid, signal),
        ["poll"] => Ok(poll::send_poll_data(poll_data)),
        _ => {
            eprintln!("Unknown cmd: {cmd}");
            Err(format!("Unknown cmd: {cmd}"))
        }
    }
}
