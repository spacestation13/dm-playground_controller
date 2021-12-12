//! Handles subprocess calling and buffering of stdout and stdin

use std::{fs, path};
use subprocess::Exec;

/// Takes in x y z
///
///  Returns: Ok() if the unzip was successful, otherwise an Err()
pub fn process(path: String) -> Result<String, String> {
    let path_proper = path::Path::new(&path);

    let file_name = path_proper.file_stem().unwrap().to_str().unwrap(); // 514.1571_byond
    let major = &file_name[0..3]; // 514
    let minor = &file_name[4..8]; // 1571

    let tmp_path = format!("/tmp/{}/{}", major, minor);
    fs::create_dir_all(&tmp_path)
        .unwrap_or_else(|_| panic!("Couldn't create BYOND dir: {}", tmp_path));

    let exit_status = Exec::cmd("umount").arg(path_proper).join().unwrap();

    Ok("OK".into())
}
