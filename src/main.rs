#![deny(clippy::all)]
#![deny(clippy::pedantic)]

#[macro_use]
mod helpers;
mod poll;
mod process;
mod signal;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
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
            let mut read_buf = [0u8; 256];
            let poll_data: Arc<Mutex<Vec<PollData>>> = Arc::new(Mutex::new(vec![]));

            debug!("Receiving data on serial connection.");
            loop {
                // read data from serial port in chunks until null terminator
                loop {
                    match port.read(&mut read_buf) {
                        Ok(n) => {
                            let data = &read_buf[..n];
                            // if the chunk contains a null byte take up to it and break, otherwise - accumulate
                            if let Some(pos) = data.iter().position(|&b| b == 0x00) {
                                serial_buf.extend_from_slice(&data[..pos]);
                                break;
                            }
                            serial_buf.extend_from_slice(data);
                        }
                        Err(e)
                            if e.kind() == io::ErrorKind::TimedOut
                                || e.kind() == io::ErrorKind::UnexpectedEof => {}
                        Err(e) => panic!("IO error when reading buffer: {e}"),
                    }
                }

                match process_cmds(&serial_buf, &poll_data) {
                    Ok(s) => {
                        port.write_all(format!("{}OK\0", &s).as_bytes())
                            .expect("Error writing to serial");
                    }
                    Err(e) => {
                        port.write_all(
                            format!("{}\nERR\0", BASE64.encode(e.as_bytes())).as_bytes(),
                        )
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
