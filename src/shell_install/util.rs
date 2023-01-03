use {
    crate::errors::*,
    std::{
        fs::{self, OpenOptions},
        io::{BufRead, BufReader, Write},
        path::Path,
    },
};
pub fn file_contains_line(path: &Path, searched_line: &str) -> Result<bool, ShellInstallError> {
    let file = fs::File::open(path)
        .context(&|| format!("opening {path:?}"))?;
    for line in BufReader::new(file).lines() {
        let line = line.context(&|| format!("reading line in {path:?}"))?;
        if line == searched_line {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn append_to_file<S: AsRef<str>>(path: &Path, content: S) -> Result<(), ShellInstallError> {
    let mut shellrc = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .context(&|| format!("opening {path:?} for append"))?;
    shellrc.write_all(content.as_ref().as_bytes())
        .context(&|| format!("writing in {path:?}"))?;
    Ok(())
}
