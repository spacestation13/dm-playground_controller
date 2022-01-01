//! Handles subprocess calling and buffering of stdout and stdin

use crate::{PollData, PollType};

use base64::{decode, encode};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use subprocess::{Exec, ExitStatus, Redirection};

#[derive(std::cmp::PartialEq)]
enum EnvParserState {
    Key,
    Value,
}

/// Takes in base64 process, args, and env vars data
///
///  Returns: Result
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
        let mut comms = proc.communicate_start(None);

        // Loop the process inside the thread
        loop {
            match proc.poll() {
                // If the process has exited
                Some(status) => {
                    let comm_data = comms.read_string().expect("Proc comms error on exit");
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
                    let comm_data = comms.read_string().expect("Proc comms error");
                    push_possible_output(comm_data, &poll_data);

                    // How long we sleep inside the thread to check if exited or more poll data
                    thread::sleep(Duration::new(0, 100_000));
                }
            }
        }
    });

    Ok(format!("{}\n", pid))
}

fn push_possible_output(
    (stdout, stderr): (Option<String>, Option<String>),
    poll_data: &Arc<Mutex<Vec<PollData>>>,
) {
    let out_dat = stdout.expect("Stdout pipe is closed");
    let err_dat = stderr.expect("Stderr pipe is closed");
    //Avoid locking if there's no incoming data
    if out_dat.is_empty() && err_dat.is_empty() { return }

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
