//! Handles poll data being sent through the serial connection
//!
use crate::PollData;
use base64::encode;
use serialport::SerialPort;

/// Sends the poll data through the serial connection
pub fn send_poll_data(
    port: &mut (impl SerialPort + ?Sized),
    data: &[PollData],
) -> Result<String, String> {
    for dat in data {
        if let Err(e) = port.write_all(encode(format!("{} {}\n", &dat.typ, &dat.data)).as_bytes()) {
            return Err(format!("Error writing to serial during poll send: {}", e));
        }
    }
    Ok("OK".into())
}
