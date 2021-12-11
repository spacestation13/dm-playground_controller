#[macro_use]
mod helpers;
mod signal;
mod unzip;

use std::io;
use std::time::Duration;

use base64::{encode, decode};

fn main() {
    // Open serial connection on /dev/ttyS2, baud rate is chosen for <1ms latency
    let port = serialport::new("/dev/ttyS2", 19_200)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            debug!("Receiving data on serial connection.");
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(n) => {
                        let res = process_cmds(&serial_buf[..n]);
                        //TODO: send OK if res is Ok
                        match res {
                            Ok(_) => {

                            },
                            Err(e) => {
                                port.write_fmt(format_args!("{}\nERR\0", encode(&e))).unwrap();
                            }
                        }
                    },
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
/// - `u in_zip_path` - Unzip the BYOND zip file at the given path
/// - `r process_name args env_vars` - Run the specified process with the given arguments and environment variables
/// - `s pid signal` - Send the given signal to the given pid
/// - `p` - Poll for data, sends back (p pid data\n)* and/or (o pid stdout\n)* with OK for end of data
/// - `q` - Quit
fn process_cmds(serial_buf: &[u8]) -> Result<(), String> {
    // Tokenize and parse the command
    let cmd = String::from_utf8_lossy(serial_buf);
    let cmd_tokens: Vec<&str> = cmd.split_whitespace().collect();
    match cmd_tokens.as_slice() {
        ["u", in_zip_path] => unzip::unzip(String::from_utf8(decode(in_zip_path).expect("Invalid b64")).unwrap()),
        ["r", process_name, args, env_vars] => unimplemented!(),
        ["s", pid, signal] => signal::send_signal(pid, signal),
        ["p", pid] => unimplemented!(),
        ["q"] => unimplemented!(),
        _ => {
            eprintln!("Unknown cmd: {}", cmd);
            Err(format!("Unknown cmd: {}", cmd))
        }
    }
}
