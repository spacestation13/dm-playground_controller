use std::str::FromStr;

use nix::sys::signal;

/// Send the specified signal to the process with the specified pid.
/// Returns Ok if the signal was sent, or an Err if the signal could not be sent.
pub fn send_signal(pid: &&str, signal: &&str) -> Result<String, String> {
    let sig = signal::Signal::from_str(signal).expect("Malformed signal number");

    match signal::kill(
        nix::unistd::Pid::from_raw(pid.parse::<i32>().expect("Malformed pid")),
        sig,
    ) {
        Ok(_) => Ok("OK\n".into()),
        Err(e) => Err(format!(
            "Error sending signal {} to pid {}: {}\n",
            sig, pid, e
        )),
    }
}
