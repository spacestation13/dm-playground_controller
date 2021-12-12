//! Handles poll data being sent through the serial connection

use crate::PollData;

use base64::encode;
use serialport::SerialPort;
use std::{cell::RefCell, rc::Rc};

/// Sends the poll data through the serial connection
pub fn send_poll_data(
    port: &mut (impl SerialPort + ?Sized),
    data: &Rc<RefCell<Vec<PollData>>>,
) -> Result<String, String> {
    let poll_data = &*data.borrow();

    for dat in poll_data.iter() {
        if let Err(e) = port.write_all(encode(format!("{} {}\n", &dat.typ, &dat.data)).as_bytes()) {
            return Err(format!("Error writing to serial during poll send: {}\n", e));
        }
    }
    Ok("".into())
}
