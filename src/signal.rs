use nix::sys::signal;
use phf::phf_map;

/// Send the specified signal to the process with the specified pid.
/// Returns Ok if the signal was sent, or an Err if the signal could not be sent.
pub fn send(pid: &&str, signal: &&str) -> Result<String, String> {
    let signal_str = SIGNALS.get(signal).expect("Malformed signal number");
    let sig = signal_str.parse::<signal::Signal>().expect("Malformed signal name");

    match signal::kill(
        nix::unistd::Pid::from_raw(pid.parse::<i32>().expect("Malformed pid")),
        sig,
    ) {
        Ok(()) => Ok(String::new()),
        Err(e) => Err(format!("Error sending signal {sig} to pid {pid}: {e}")),
    }
}

/// Generated from running `kill -l` on the image
static SIGNALS: phf::Map<&'static str, &'static str> = phf_map! {
    "1" => "SIGHUP",
    "2" => "SIGINT",
    "3" => "SIGQUIT",
    "4" => "SIGILL",
    "5" => "SIGTRAP",
    "6" => "SIGABRT",
    "7" => "SIGBUS",
    "8" => "SIGFPE",
    "9" => "SIGKILL",
    "10" => "SIGUSR1",
    "11" => "SIGSEGV",
    "12" => "SIGUSR2",
    "13" => "SIGPIPE",
    "14" => "SIGALRM",
    "15" => "SIGTERM",
    "16" => "SIGSTKFLT",
    "17" => "SIGCHLD",
    "18" => "SIGCONT",
    "19" => "SIGSTOP",
    "20" => "SIGTSTP",
    "21" => "SIGTTIN",
    "22" => "SIGTTOU",
    "23" => "SIGURG",
    "24" => "SIGXCPU",
    "25" => "SIGXFSZ",
    "26" => "SIGVTALRM",
    "27" => "SIGPROF",
    "28" => "SIGWINCH",
    "29" => "SIGPOLL",  // No entry in Signal
    "30" => "SIGPWR",
    "31" => "SIGSYS",
    "34" => "SIGRTMIN", // No entry in Signal
    "64" => "SIGRTMAX", // No entry in Signal
};
