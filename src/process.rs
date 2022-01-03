//! Handles subprocess calling and buffering of stdout and stdin

use crate::{PollData, PollType};

use base64::{decode, encode};
use std::{
    cell::{RefCell, RefMut},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use subprocess::{Communicator, Exec, ExitStatus, Redirection};

#[derive(std::cmp::PartialEq)]
enum EnvParserState {
    Key,
    Value,
}

/// Data returned from a Communicator
struct CommData {
    stdout: Option<String>,
    stderr: Option<String>,
}

impl From<CommData> for (Option<String>, Option<String>) {
    fn from(comm: CommData) -> (Option<String>, Option<String>) {
        let CommData { stdout, stderr } = comm;
        (stdout, stderr)
    }
}

/// Takes in base64 process, args, and env vars data to run a process
///
///  Returns: The pid of the created process
pub fn process(
    b_process: &&str,
    b_args: &&str,
    b_env_vars: &&str,
    poll_data_main: &Arc<Mutex<Vec<PollData>>>,
) -> Result<String, String> {
    let process = match decode(b_process) {
        Ok(dec_vec) if dec_vec.is_empty() => "".into(),
        Ok(dec_vec) => String::from_utf8(dec_vec).expect("Invalid UTF8 for exec path"),
        Err(e) => return Err(format!("Error decoding exec path: {}", e)),
    };

    let args = match decode(b_args) {
        Ok(dec_vec) if dec_vec.is_empty() => "".into(),
        Ok(dec_vec) => String::from_utf8(dec_vec).expect("Invalid UTF8 for exec args"),
        Err(e) => return Err(format!("Error decoding exec args: {}", e)),
    };

    let raw_env_vars = match decode(b_env_vars) {
        Ok(dec_vec) if dec_vec.is_empty() => "".into(),
        Ok(dec_vec) => String::from_utf8(dec_vec).expect("Invalid UTF8 for exec env args"),
        Err(e) => return Err(format!("Error decoding exec env vars: {}", e)),
    };

    // Handle environment vars parsing into tuples
    // `VAR1=VAL1;VAR2=VAL2;`
    let mut tmpkey = String::with_capacity(30);
    let mut tmpval = String::with_capacity(30);
    let mut env_vars: Vec<(String, String)> = vec![];
    let mut state: EnvParserState = EnvParserState::Key;
    let mut skip = false;

    /// Depending on state, adds the given char to the proper tuple portion
    fn add_char(char: &char, state: &EnvParserState, tmpkey: &mut String, tmpval: &mut String) {
        if *state == EnvParserState::Key {
            tmpkey.push(*char);
        } else {
            tmpval.push(*char);
        }
    }

    // Process environmental vars
    for char in raw_env_vars.chars() {
        // This is triggered if we escape
        if skip {
            add_char(&char, &state, &mut tmpkey, &mut tmpval);
            continue;
        }

        match char {
            '\\' => {
                // escape
                skip = true
            }
            '=' => {
                // switch state
                if state != EnvParserState::Key {
                    return Err("Env arg has several values".to_string());
                }
                state = EnvParserState::Value;
            }
            ';' => {
                // signal switch into next
                if state != EnvParserState::Value {
                    return Err("Env arg is missing value".to_string());
                }
                state = EnvParserState::Key;

                env_vars.push((tmpkey, tmpval));

                tmpkey = String::with_capacity(30);
                tmpval = String::with_capacity(30);
            }
            // otherwise add to the block in our current state
            _ => add_char(&char, &state, &mut tmpkey, &mut tmpval),
        }
    }
    if state != EnvParserState::Key {
        return Err("Env args are unterminated".to_string());
    }

    let poll_data = poll_data_main.clone();

    let mut proc = Exec::cmd(process)
        .arg(args)
        .env_extend(&env_vars)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Pipe)
        .popen()
        .expect("Failed to start process");

    let pid = proc.pid().unwrap(); // Must exist for a newly opened process
    thread::spawn(move || {
        let comms = RefCell::new(
            proc.communicate_start(None)
                .limit_time(Duration::new(0, 100_000)),
        );

        // Loop the process inside the thread
        loop {
            match proc.poll() {
                // If the process has exited
                Some(status) => {
                    let comm_data = get_comm_data(comms.borrow_mut());
                    push_possible_output(comm_data, &poll_data);

                    // Push the pid and exit status since we've exited
                    //0-255: Exit codes
                    //256: Undetermined
                    //257-inf: Signaled
                    let exit_code = match status {
                        ExitStatus::Exited(code) => code,
                        ExitStatus::Undetermined => 256,
                        ExitStatus::Signaled(signal) => 256 + (signal as u32),
                        ExitStatus::Other(what) => panic!("Unknown ExitStatus: {}", what),
                    };
                    poll_data.lock().unwrap().push(PollData {
                        typ: PollType::PidExit,
                        data: format!("{} {}", pid, exit_code),
                    });

                    break;
                }
                // If the process is still running
                None => {
                    let comm_data = get_comm_data(comms.borrow_mut());
                    push_possible_output(comm_data, &poll_data);

                    // How long we sleep inside the thread to check if exited or more poll data
                    thread::sleep(Duration::new(0, 100_000));
                }
            }
        }
    });

    Ok(format!("{}\n", pid))
}

fn get_comm_data(mut comms: RefMut<Communicator>) -> CommData {
    match comms.read_string() {
        Ok(data) => {
            // Just drop comms and give eof'd data
            drop(comms);
            CommData {
                stdout: data.0,
                stderr: data.1,
            }
        }
        Err(comm_error) => {
            // Ignore error and give partial (non-eof) data if it exists
            let data = comm_error.capture;
            drop(comms);
            CommData {
                stdout: data.0.map(|dat| String::from_utf8_lossy(&dat).into_owned()),
                stderr: data.1.map(|dat| String::from_utf8_lossy(&dat).into_owned()),
            }
        }
    }
}

fn push_possible_output(data: CommData, poll_data: &Arc<Mutex<Vec<PollData>>>) {
    let out_dat = data.stdout.unwrap_or_default();
    let err_dat = data.stderr.unwrap_or_default();

    //Avoid locking if there's no incoming data
    if out_dat.is_empty() && err_dat.is_empty() {
        return;
    }

    let mut poll_lock = poll_data.lock().unwrap();
    if !out_dat.is_empty() {
        poll_lock.push(PollData {
            typ: PollType::Stdout,
            data: encode(out_dat),
        });
    }
    if !err_dat.is_empty() {
        poll_lock.push(PollData {
            typ: PollType::Stderr,
            data: encode(err_dat),
        });
    }
}
