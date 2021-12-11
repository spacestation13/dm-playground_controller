//! Handles unzipping functionality for sending files around

use std::fs;
use std::io;

// TODO: extracts it to a uniqe folder in /tmp
///  Takes a zip file from a given [path], extracts it to a unique folder in /tmp
/// 
///  Returns: Ok() if the unzip was successful, otherwise an Err()
fn unzip(path: &str) -> Result<(), ()> {
    let file = fs::File::open(&path).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        // If we have a directory, create it
        if (&*file.name()).ends_with('/') {
            debug!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath).unwrap();
        }
        // Else we have a normal file
        else {
            debug!("File {} extracted to \"{}\"", i, outpath.display());
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        // If nix, fix permissions of extracted file path
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                if let Err(e) = fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)) {
                    eprintln!(
                        "Error while setting permissions on path: {}, mode: {}",
                        &outpath
                            .to_str()
                            .expect("Unzip outpath contains non-unicode chars"),
                        e
                    );
                }
            }
        }
    }

    Ok(())
}
