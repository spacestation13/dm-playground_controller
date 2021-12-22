//! Handles subprocess calling and buffering of stdout and stdin

use crate::{PollData, PollType};

use base64::decode;
use std::{cell::RefCell, sync::Arc};
use subprocess::Exec;

#[derive(std::cmp::PartialEq)]
enum EnvParserState {
    Key,
    Value,
}

/// Takes in x y z
///
///  Returns: Ok() if the unzip was successful, otherwise an Err()
pub async fn process(
    b_process: &&str,
    b_args: &&str,
    b_env_vars: &&str,
    poll_data: &Arc<RefCell<Vec<PollData>>>,
) -> Result<String, String> {
    let process = match decode(b_process) {
        Ok(dec_process) => String::from_utf8(dec_process).expect("Invalid UTF8 for exec path"),
        Err(e) => return Err(format!("Error decoding exec path: {}\n", e.to_string())),
    };

    let args = match decode(b_args) {
        Ok(dec_args) => String::from_utf8(dec_args).expect("Invalid UTF8 for exec args"),
        Err(e) => return Err(format!("Error decoding exec args: {}\n", e.to_string())),
    };

    let raw_env_vars = match decode(b_env_vars) {
        Ok(dec_env_vars) => {
            String::from_utf8(dec_env_vars).expect("Invalid UTF8 for exec env vars")
        }
        Err(e) => return Err(format!("Error decoding exec env vars: {}\n", e.to_string())),
    };

    // `VAR1=VAL1;VAR2=VAL2;`
    let mut tmpkey = String::with_capacity(30);
    let mut tmpval = String::with_capacity(30);
    let mut env_vars: Vec<(String, String)> = vec![];
    let mut state: EnvParserState = EnvParserState::Key;
    let mut skip = false;

    let add_char = |char: char| {
        if state == EnvParserState::Key {
            tmpkey.push(char);
        } else {
            tmpval.push(char);
        }
    };

    for char in raw_env_vars.chars() {
        if skip {
            add_char(char);
            continue;
        }

        match char {
            '\\' => skip = true,
            '=' => {
                if state != EnvParserState::Key {
                    return Err("Env arg has several values".to_string());
                }
                state = EnvParserState::Value;
            }
            ';' => {
                if state != EnvParserState::Value {
                    return Err("Env arg is missing value".to_string());
                }
                state = EnvParserState::Key;

                env_vars.push((tmpkey, tmpval));

                tmpkey = String::with_capacity(30);
                tmpval = String::with_capacity(30);
            }
            _ => add_char(char),
        }
    }

    let mut env_vars = vec![];
    let split_env_vars = raw_env_vars.split(';');
    for pair in split_env_vars {
        // `VAR1=VAL1`
        let mut pair_sp = pair.split('=');
        let var = pair_sp.next().expect("Malformed env arg variable");
        let val = pair_sp.last().expect("Malformed env arg value");
        env_vars.push((var, val))
    }

    tokio::spawn(async move {});

    // Blocking currently
    let proc_capture = Exec::cmd(process)
        .arg(args)
        .env_extend(&env_vars)
        .capture()
        .expect("Process failure");

    let stderr: PollData = PollData {
        typ: PollType::Stderr,
        data: proc_capture.stdout_str(),
    };

    poll_data.borrow_mut().push(stderr);

    Ok("OK\n".into())
}
