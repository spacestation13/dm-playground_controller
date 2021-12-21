//! Handles poll data being sent through the serial connection

use crate::PollData;

use std::{cell::RefCell, sync::Arc};

/// Sends the poll data through the serial connection
pub fn send_poll_data(data: &Arc<RefCell<Vec<PollData>>>) -> Result<String, String> {
    let poll_data = &*data.borrow();

    Ok(poll_data
        .iter()
        .map(|dat| format!("{} {}\n", dat.typ.to_string(), &dat.data))
        .collect::<Vec<String>>()
        .concat())
}
