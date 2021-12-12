use serialport::SerialPort;
use base64::encode;
use crate::PollData;


/// Sends the poll data through the serial connection
pub fn send_poll_data(port: &mut (impl SerialPort + ?Sized), data: &[PollData]) -> Result<String, String> {
    for dat in data {
        if let Err(e) = port.write_all(encode(format!("{} {}\n", &dat.typ, &dat.data)).as_bytes()) {
            return Err(format!("Error writing to serial during poll send: {}", e));
        }
    }
    Ok("OK".into())
}