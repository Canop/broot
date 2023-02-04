use {
    include_dir::{Dir, include_dir},
    std::{
        fs,
        io,
        path::Path,
    },
};

static DEFAULT_CONF_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/resources/default-conf");

pub fn write_default_conf_in(dir: &Path) -> Result<(), io::Error> {
    info!("writing default conf in {:?}", dir);
    if dir.exists() {
        if !dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("{dir:?} isn't a directory"),
            ));
        }
    } else {
        fs::create_dir_all(dir)?;
    }
    for file in DEFAULT_CONF_DIR.files() {
        let dest_path = dir.join(file.path());
        if dest_path.exists() {
            warn!("not overwriting {:?}", dest_path);
        } else {
            info!("writing file {:?}", file.path());
            fs::write(dest_path, file.contents())?;
        }
    }
    Ok(())
}
