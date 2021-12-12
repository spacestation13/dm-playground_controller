//! Handles subprocess calling and buffering of stdout and stdin

use crate::PollData;

use std::{cell::RefCell, fs, path, rc::Rc};
use subprocess::Exec;

/// Takes in x y z
///
///  Returns: Ok() if the unzip was successful, otherwise an Err()
pub fn process(
    process: &&str,
    _args: &&str,
    _env_vars: &&str,
    poll_data: &Rc<RefCell<Vec<PollData>>>,
) -> Result<String, String> {
    let path_proper = path::Path::new(&process);

    let file_name = path_proper.file_stem().unwrap().to_str().unwrap(); // 514.1571_byond
    let major = &file_name[0..3]; // 514
    let minor = &file_name[4..8]; // 1571

    let tmp_path = format!("/tmp/{}/{}", major, minor);
    fs::create_dir_all(&tmp_path)
        .unwrap_or_else(|_| panic!("Couldn't create BYOND dir: {}", tmp_path));

    let _exit_status = Exec::cmd("umount").arg(path_proper).join().unwrap();
    let dat: PollData = PollData {
        typ: "".into(),
        data: "".into(),
    };
    poll_data.borrow_mut().push(dat);

    Ok("OK".into())
}
