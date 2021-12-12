use serialport::SerialPort;
use base64::{encode};
use crate::PollData;


/// Sends poll data through the serial connection
pub fn send_poll_data(port: &mut (impl SerialPort + ?Sized), data: &[PollData]) -> Result<String, String> {
    for dat in data {
        port.write_all(encode(&dat.typ).as_bytes()).expect("Failure writing");
    }
    Ok("OK".into())
}