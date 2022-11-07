//! Handles poll data being sent through the serial connection

use crate::PollData;

use std::sync::{Arc, Mutex};

/// Sends the poll data through the serial connection
pub fn send_poll_data(data: &Arc<Mutex<Vec<PollData>>>) -> std::string::String {
    let poll_data = &mut *data.lock().unwrap();
    let poll_data_str = poll_data
        .iter()
        .map(|dat| format!("{} {} {}\n", dat.typ, &dat.pid, &dat.data))
        .collect::<Vec<String>>()
        .concat();
    poll_data.clear();
    poll_data_str
}
