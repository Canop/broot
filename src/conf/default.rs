use {
    include_dir::{Dir, DirEntry, include_dir},
    std::{
        fs,
        io,
        path::Path,
    },
};

static DEFAULT_CONF_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/resources/default-conf");

/// Write the default configuration files in the destination directory, not
/// overwriting existing ones
pub fn write_default_conf_in(dir: &Path) -> Result<(), io::Error> {
    info!("writing default conf in {:?}", dir);
    if dir.exists() {
        if !dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("{dir:?} isn't a directory"),
            ));
        }
    }
    let mut files = Vec::new();
    find_files(&DEFAULT_CONF_DIR, &mut files);
    for file in files {
        let dest_path = dir.join(file.path());
        if dest_path.exists() {
            warn!("not overwriting {:?}", dest_path);
        } else {
            if let Some(dir) = dest_path.parent() {
                if !dir.exists() {
                    fs::create_dir_all(dir)?;
                }
            };
            info!("writing file {:?}", file.path());
            fs::write(dest_path, file.contents())?;
        }
    }
    Ok(())
}

fn find_files<'d>(dir: &'d Dir<'d>, files: &mut Vec<&'d include_dir::File<'d>>) {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(sub_dir) => {
                find_files(sub_dir, files);
            }
            DirEntry::File(file) => {
                files.push(file);
            }
        }
    }
}

/// Check that all the files in the default_conf directory are valid
/// configuration files
#[test]
fn check_default_conf_files() {
    use crate::conf::*;
    let mut files = Vec::new();
    find_files(&DEFAULT_CONF_DIR, &mut files);
    for file in files {
        println!("Checking {}", file.path().display());
        let file_content = std::str::from_utf8(file.contents()).unwrap();
        SerdeFormat::read_string::<Conf>(file.path(), file_content).unwrap();
    }
}
