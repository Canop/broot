use {
    crate::{
        app::*,
        errors::ProgramError,
    },
    std::{
        fs::{
            File,
            OpenOptions,
        },
        io::Write,
    },
};

/// Intended to verbs, this function writes the passed string to the file
/// provided to broot with `--verb-output`, creating a new line if the
/// file is not empty.
pub fn verb_write(
    con: &AppContext,
    line: &str,
) -> Result<CmdResult, ProgramError> {
    let Some(path) = &con.launch_args.verb_output else {
        return Ok(CmdResult::error("No --verb-output provided".to_string()));
    };
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    if file.metadata().map(|m| m.len() > 0).unwrap_or(false) {
        writeln!(file)?;
    }
    write!(file, "{}", line)?;
    Ok(CmdResult::Keep)
}

/// Remove the content of the file provided to broot with `--verb-output`.
pub fn verb_clear_output(con: &AppContext) -> Result<CmdResult, ProgramError> {
    let Some(path) = &con.launch_args.verb_output else {
        return Ok(CmdResult::error("No --verb-output provided".to_string()));
    };
    File::create(path)?;
    Ok(CmdResult::Keep)
}
