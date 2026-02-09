//! Handles subprocess calling and buffering of stdout and stdin

use crate::{PollData, PollType};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::io::ErrorKind;
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

/// Takes in base64 process, args, and env vars data to run a process
///
///  Returns: The pid of the created process
pub fn process(
    b_process: &&str,
    b_args: &&str,
    b_env_vars: &&str,
    poll_data_main: &Arc<Mutex<Vec<PollData>>>,
) -> Result<String, String> {
    let process = match BASE64.decode(b_process) {
        Ok(dec_vec) if dec_vec.is_empty() => String::new(),
        Ok(dec_vec) => String::from_utf8(dec_vec).expect("Invalid UTF8 for exec path"),
        Err(e) => return Err(format!("Error decoding exec path: {e}")),
    };

    let raw_args = match BASE64.decode(b_args) {
        Ok(dec_vec) if dec_vec.is_empty() => String::new(),
        Ok(dec_vec) => String::from_utf8(dec_vec).expect("Invalid UTF8 for exec args"),
        Err(e) => return Err(format!("Error decoding exec args: {e}")),
    };
    let args = raw_args.split('\0').collect::<Vec<&str>>();

    let raw_env_vars = match BASE64.decode(b_env_vars) {
        Ok(dec_vec) if dec_vec.is_empty() => String::new(),
        Ok(dec_vec) => String::from_utf8(dec_vec).expect("Invalid UTF8 for exec env args"),
        Err(e) => return Err(format!("Error decoding exec env vars: {e}")),
    };

    // Handle environment vars parsing into tuples
    // `VAR1=VAL1;VAR2=VAL2;`
    let mut tmpkey = String::with_capacity(30);
    let mut tmpval = String::with_capacity(30);
    let mut env_vars: Vec<(String, String)> = vec![];
    let mut state: EnvParserState = EnvParserState::Key;
    let mut skip = false;

    // Process environmental vars
    for char in raw_env_vars.chars() {
        // This is triggered if we escape
        if skip {
            add_char(char, &state, &mut tmpkey, &mut tmpval);
            continue;
        }

        match char {
            '\\' => {
                // escape
                skip = true;
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
            _ => add_char(char, &state, &mut tmpkey, &mut tmpval),
        }
    }
    if state != EnvParserState::Key {
        return Err("Env args are unterminated".to_string());
    }

    let poll_data = poll_data_main.clone();

    let mut proc = match Exec::cmd(process)
        .args(args.as_slice())
        .env_extend(&env_vars)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Pipe)
        .popen()
    {
        Err(e) => return Err(format!("Failed to create process: {e}")),
        Ok(proc) => proc,
    };

    let pid = proc.pid().unwrap(); // Must exist for a newly opened process
    thread::spawn(move || {
        let mut comms = proc
            .communicate_start(None)
            .limit_time(Duration::from_millis(10));

        // Loop the process inside the thread
        loop {
            let comm_data = {
                match comms.read_string() {
                    // Just drop comms and give eof'd data
                    Ok(data) => (data.0, data.1),
                    Err(comm_error) if comm_error.kind() == ErrorKind::TimedOut => {
                        // Ignore 'error' and give partial (non-eof) data if it exists
                        let data = comm_error.capture;
                        (
                            data.0.map(|dat| String::from_utf8_lossy(&dat).into_owned()),
                            data.1.map(|dat| String::from_utf8_lossy(&dat).into_owned()),
                        )
                    }
                    Err(e) => panic!("Error while reading comms: {e}"),
                }
            };
            push_possible_output(comm_data, pid, &poll_data);

            // Have we exited? We'll need to push the PID and exit data
            if let Some(status) = proc.poll() {
                //0-255: Exit codes
                //256: Undetermined
                //257-inf: Signaled
                let exit_code = match status {
                    ExitStatus::Exited(code) => code,
                    ExitStatus::Undetermined => 256,
                    ExitStatus::Signaled(signal) => 256 + u32::from(signal),
                    ExitStatus::Other(what) => panic!("Unknown ExitStatus: {what}"),
                };
                poll_data.lock().unwrap().push(PollData {
                    typ: PollType::PidExit,
                    pid,
                    data: exit_code.to_string(),
                });

                break;
            }
        }
    });

    Ok(format!("{pid}\n"))
}

fn push_possible_output(
    (stdout, stderr): (Option<String>, Option<String>),
    pid: u32,
    poll_data: &Arc<Mutex<Vec<PollData>>>,
) {
    let out_dat = stdout.unwrap_or_default();
    let err_dat = stderr.unwrap_or_default();

    //Avoid locking if there's no incoming data
    if out_dat.is_empty() && err_dat.is_empty() {
        return;
    }

    let mut poll_lock = poll_data.lock().unwrap();
    if !out_dat.is_empty() {
        poll_lock.push(PollData {
            typ: PollType::Stdout,
            pid,
            data: BASE64.encode(&out_dat),
        });
    }
    if !err_dat.is_empty() {
        poll_lock.push(PollData {
            typ: PollType::Stderr,
            pid,
            data: BASE64.encode(&err_dat),
        });
    }
}

/// Depending on `EnvParserState`, adds `char` to the proper tuple portion
fn add_char(char: char, state: &EnvParserState, tmpkey: &mut String, tmpval: &mut String) {
    if *state == EnvParserState::Key {
        tmpkey.push(char);
    } else {
        tmpval.push(char);
    }
}
