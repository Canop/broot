use {
    crate::{
        app::*,
        display::W,
        errors::ProgramError,
    },
    crokey::crossterm::{
        terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
    },
    std::{
        fs::{File, OpenOptions},
        io::Write,
    },
};

/// Intended to verbs, this function writes the passed string to the file
/// provided to broot with `--verb-output`, creating a new line if the
/// file is not empty.
pub fn verb_write_output(
    con: &AppContext,
    line: &str,
) -> Result<CmdResult, ProgramError> {
    let Some(path) = &con.launch_args.verb_output else {
        return Ok(CmdResult::error("No --verb-output provided".to_string()));
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    if file.metadata().map(|m| m.len() > 0).unwrap_or(false) {
        writeln!(file)?;
    }
    write!(file, "{}", line)?;
    Ok(CmdResult::Keep)
}

/// Remove the content of the file provided to broot with `--verb-output`.
pub fn verb_clear_output(
    con: &AppContext,
) -> Result<CmdResult, ProgramError> {
    let Some(path) = &con.launch_args.verb_output else {
        return Ok(CmdResult::error("No --verb-output provided".to_string()));
    };
    File::create(path)?;
    Ok(CmdResult::Keep)
}

pub fn verb_write_stdout(
    w: &mut W,
    line: &str,
) -> Result<CmdResult, ProgramError> {
    w.queue(LeaveAlternateScreen).unwrap();
    terminal::disable_raw_mode().unwrap();
    w.flush().unwrap();
    print!("{}", line);
    terminal::enable_raw_mode().unwrap();
    w.queue(EnterAlternateScreen).unwrap();
    w.flush().unwrap();
    Ok(CmdResult::Keep)
}
