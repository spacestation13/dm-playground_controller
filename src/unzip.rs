//! Handles unzipping functionality for BYOND installation zips

use std::{fs, io, os, path};

///  Takes a byond install zip file from a given [path], extracts it to /tmp/major/minor
///
///  Returns: Ok() if the unzip was successful, otherwise an Err()
pub fn unzip(path: String) -> Result<(), String> {
    let path_proper = path::Path::new(&path);
    let file = fs::File::open(path_proper).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    if archive.is_empty() {
        return Err("Empty BYOND archive".to_string());
    }

    let file_name = path_proper.file_stem().unwrap().to_str().unwrap(); // 514.1571_byond
    let major = &file_name[0..3]; // 514
    let minor = &file_name[4..8]; // 1571

    let tmp_path = format!("/tmp/{}/{}", major, minor);
    fs::create_dir_all(&tmp_path)
        .unwrap_or_else(|_| panic!("Couldn't create BYOND dir: {}", tmp_path));

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
            use os::unix::fs::PermissionsExt;

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
