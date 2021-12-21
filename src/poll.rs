//! Handles poll data being sent through the serial connection

use crate::PollData;

use base64::encode;
use serialport::SerialPort;
use std::{cell::RefCell, sync::Arc};

/// Sends the poll data through the serial connection
pub fn send_poll_data(
    port: &mut (impl SerialPort + ?Sized),
    data: &Arc<RefCell<Vec<PollData>>>,
) -> Result<String, String> {
    let poll_data = &*data.borrow();

    for dat in poll_data.iter() {
        if let Err(e) = port.write_all(
            encode(format!(
                "{} {}\n",
                //If one of the types has an utf8 emoji in it, please euthanize me -alex
                dat.typ.to_string().to_ascii_lowercase(),
                &dat.data
            ))
            .as_bytes(),
        ) {
            return Err(format!("Error writing to serial during poll send: {}\n", e));
        }
    }
    Ok("".into())
}
