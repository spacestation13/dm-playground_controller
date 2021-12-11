#[macro_use]
pub mod helpers;

pub mod unzip;

use argh::FromArgs;
use std::io;
use std::time::Duration;

#[derive(FromArgs)]
/// Arguments for dmp-c
struct Args {
    /// device path to a serial port
    #[argh(option, default = "String::from(\"/dev/ttyS2\")")]
    port: String,
}

fn main() {
    let arguments: Args = argh::from_env();
    let port_name = &arguments.port;

    // Open serial connection on the given port, baud rate is chosen for <1ms latency
    let port = serialport::new(port_name, 19_200)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            debug!("Receiving data on {}:", &port_name);
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(n) => process_cmds(&serial_buf[..n]),
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (), // Ignore timeouts
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        // If the port could not be opened, print an error and exit
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            std::process::exit(1);
        }
    }
}

/// Process the serial buffer and parse commands therein
/// Commands:
///  - 'u in_zip_path' - Unzip the BYOND zip file at the given path
///  - 'r process_name args env_vars' - Run the specified process with the given arguments and environment variables
///  - 's signal pid' - Send the given signal to the given pid
///  - 'g file_data result_loc' - Grab the given file data and store it to be send to the given file path
///  - 'w result_loc' - Write the data associated with the given file path that we stored earlier
///  - 'q' - Quit
fn process_cmds(serial_buf: &[u8]) {
    // Tokenize and parse the command
    let cmd = String::from_utf8_lossy(serial_buf);
    let cmd_tokens: Vec<&str> = cmd.split_whitespace().collect();
    match cmd_tokens.as_slice() {
        ["u", in_zip_path] => unzip::unzip(in_zip_path).expect("Unzip failed"),
        ["r", process_name, args, env_vars] => unimplemented!(),
        ["s", signal, pid] => unimplemented!(),
        ["g", file_data, result_loc] => unimplemented!(),
        ["w", result_loc] => unimplemented!(),
        ["q"] => unimplemented!(),
        _ => eprintln!("Unknown command: {}", cmd),
    }
}
