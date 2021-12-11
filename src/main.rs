#[macro_use]
pub mod helpers;

pub mod unzip;

use argh::FromArgs;
use std::io::{self, Write};
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
                    Ok(t) => io::stdout().write_all(&serial_buf[..t]).unwrap(),
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
