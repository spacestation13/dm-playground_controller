#[macro_use]
mod helpers;
mod poll;
mod process;
mod signal;

use std::{cell::RefCell, io, sync::Arc, time::Duration};

use base64::encode;
use serialport::SerialPort;

#[derive(strum_macros::Display)]
pub enum PollType {
    #[strum(to_string = "pidexit")]
    PidExit,
    #[strum(to_string = "stdout")]
    Stdout,
    #[strum(to_string = "stderr")]
    Stderr,
}

pub struct PollData {
    typ: PollType,
    data: String,
}

#[tokio::main]
async fn main() {
    // Open serial connection on /dev/ttyS2, max baud rate
    let port = serialport::new("/dev/ttyS2", 115_200)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(mut port) => {
            port.write_all("OK\0".as_bytes())
                .expect("Error writing to serial");

            let mut serial_buf: Vec<u8> = vec![0; 5000];
            let poll_data: Arc<RefCell<Vec<PollData>>> = Arc::new(RefCell::new(vec![]));
            debug!("Receiving data on serial connection.");
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(n) => {
                        let res = process_cmds(&serial_buf[..n], &poll_data, &mut *port);
                        let result = res.await;
                        match result {
                            Ok(s) => {
                                port.write_all(format!("{}\nOK\0", &s).as_bytes())
                                    .expect("Error writing to serial");
                            }
                            Err(e) => {
                                port.write_all(format!("{}\nERR\0", encode(&e)).as_bytes())
                                    .expect("Error writing to serial");
                            }
                        }
                        port.flush().expect("Couldn't flush serial on reading end");
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (), // Ignore timeouts
                    Err(e) => eprintln!("{:?}", e),
                }
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
    poll_data: &Arc<RefCell<Vec<PollData>>>,
    port: &mut (impl SerialPort + ?Sized),
) -> Result<String, String> {
    // Tokenize and parse the command
    let cmd = String::from_utf8_lossy(serial_buf);
    let cmd_tokens: Vec<&str> = cmd.split_whitespace().collect();

    match cmd_tokens.as_slice() {
        ["run", process_name, args, env_vars] => {
            process::process(process_name, args, env_vars, poll_data).await
        }
        ["signal", pid, signal] => signal::send_signal(pid, signal),
        ["poll"] => poll::send_poll_data(port, poll_data),
        _ => {
            eprintln!("Unknown cmd: {}", cmd);
            Err(format!("Unknown cmd: {}\n", cmd))
        }
    }
}
