#[macro_use]
mod helpers;
mod poll;
mod process;
mod signal;

use std::cell::RefCell;
use std::time::Duration;
use std::{io, rc::Rc};

use base64::{decode, encode};
use serialport::SerialPort;

pub struct PollData {
    typ: String,
    data: String,
}

fn main() {
    // Open serial connection on /dev/ttyS2, max baud rate
    let port = serialport::new("/dev/ttyS2", 115_200)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 5000];
            let poll_data: Rc<RefCell<Vec<PollData>>> = Rc::new(RefCell::new(vec![]));
            debug!("Receiving data on serial connection.");
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(n) => {
                        let res = process_cmds(&serial_buf[..n], &poll_data, &mut *port);
                        //TODO: send OK if res is Ok
                        match res {
                            Ok(_) => {}
                            Err(e) => {
                                port.write_fmt(format_args!("{}\nERR\0", encode(&e)))
                                    .unwrap();
                            }
                        }
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
fn process_cmds(
    serial_buf: &[u8],
    poll_data: &Rc<RefCell<Vec<PollData>>>,
    port: &mut (impl SerialPort + ?Sized),
) -> Result<String, String> {
    // Tokenize and parse the command
    let cmd = String::from_utf8_lossy(serial_buf);
    let cmd_tokens: Vec<&str> = cmd.split_whitespace().collect();

    match cmd_tokens.as_slice() {
        ["run", process_name, args, env_vars] => {
            process::process(process_name, args, env_vars, poll_data)
        }
        ["signal", pid, signal] => signal::send_signal(pid, signal),
        ["poll"] => poll::send_poll_data(port, poll_data),
        _ => {
            eprintln!("Unknown cmd: {}", cmd);
            Err(format!("Unknown cmd: {}", cmd))
        }
    }
}
