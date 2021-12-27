#[macro_use]
mod helpers;
mod poll;
mod process;
mod signal;

use std::{
    io::{self, Read},
    sync::{Arc, Mutex},
    time::Duration,
};

use base64::encode;
use subprocess::Popen;

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
    data: String,
}

/// Holds information relating to a (possibly) open process
pub struct ProcData {
    pid: u32,
    popen: Popen,
}

#[tokio::main]
async fn main() {
    // Open serial connection on /dev/ttyS2, max baud rate
    let port = serialport::new("/dev/ttyS2", 115_200)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(mut port) => {
            // Hello message
            port.write_all("OK\0".as_bytes())
                .expect("Error writing to serial");

            let mut serial_buf: Vec<u8> = Vec::with_capacity(5000);
            let mut serial_char_buf: Vec<u8> = vec![0; 1];
            let poll_data: Arc<Mutex<Vec<PollData>>> = Arc::new(Mutex::new(vec![]));
            let running_procs: Arc<Mutex<Vec<ProcData>>> = Arc::new(Mutex::new(vec![]));

            debug!("Receiving data on serial connection.");
            loop {
                loop {
                    match port.bytes_to_read() {
                        Ok(0) => continue,
                        Ok(_) => {}
                        Err(e) => match e.kind {
                            serialport::ErrorKind::NoDevice => panic!("Serial device disconnected"),
                            serialport::ErrorKind::Io(io_error_kind) => match io_error_kind {
                                io::ErrorKind::TimedOut => continue,
                                _ => panic!("IO error when reading buffer length: {}", e),
                            },
                            _ => panic!("Unexpected error: {}", e),
                        },
                    }
                    if let Err(e) = port.read_exact(&mut serial_char_buf) {
                        match e.kind() {
                            io::ErrorKind::TimedOut => continue,
                            _ => panic!("IO error when reading buffer length: {}", e),
                        }
                    }

                    match serial_char_buf[0] {
                        0x00 => break,
                        c => serial_buf.push(c),
                    }
                }

                let res = process_cmds(&serial_buf, &poll_data, &running_procs);
                match res.await {
                    Ok(s) => {
                        port.write_all(format!("{}OK\0", &s).as_bytes())
                            .expect("Error writing to serial");
                    }
                    Err(e) => {
                        port.write_all(format!("{}ERR\0", encode(&e)).as_bytes())
                            .expect("Error writing to serial");
                    }
                }
                serial_buf.clear();
                port.flush().expect("Couldn't flush serial on reading end");
            }
        }
        // If the port could not be opened, print an error and exit
        Err(e) => {
            eprintln!("Failed to open serial connection. Error: {}", e);
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
async fn process_cmds(
    serial_buf: &[u8],
    poll_data: &Arc<Mutex<Vec<PollData>>>,
    running_procs: &Arc<Mutex<Vec<ProcData>>>,
) -> Result<String, String> {
    // Tokenize and parse the command
    let cmd = String::from_utf8_lossy(serial_buf);
    let cmd_tokens: Vec<&str> = cmd.split_whitespace().collect();

    match cmd_tokens.as_slice() {
        ["run", process_name, args, env_vars] => {
            process::process(running_procs, process_name, args, env_vars, poll_data).await
        }
        ["signal", pid, signal] => signal::send_signal(pid, signal),
        ["poll"] => poll::send_poll_data(poll_data),
        _ => {
            eprintln!("Unknown cmd: {}", cmd);
            Err(format!("Unknown cmd: {}\n", cmd))
        }
    }
}
