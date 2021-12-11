use std::str::FromStr;

use nix::sys::signal::{kill, Signal};

/// Send the specified signal to the process with the specified pid.
/// Returns Ok if the signal was sent, or an Err if the signal could not be sent.
pub fn send_signal(pid: &&str, signal: &&str) -> Result<(), String> {
    let sig = Signal::from_str(signal).expect("Malformed signal number");

    match kill(
        nix::unistd::Pid::from_raw(pid.parse::<i32>().expect("Malformed pid")),
        sig,
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!(
            "Error sending signal {} to pid {}: {}",
            sig, pid, e
        )),
    }
}
