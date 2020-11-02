use {
    std::path::Path,
};

/// from a path, build a string usable in a shell command, wrapping
///  it in quotes if necessary (and then escaping internal quotes).
/// Don't do unnecessary transformation, so that the produced string
///  is prettier on screen.
pub fn escape_for_shell(path: &Path) -> String {
    let path = path.to_string_lossy();
    if regex!(r"^[\w/.-]*$").is_match(&path) {
        path.to_string()
    } else {
        format!("'{}'", &path.replace('\'', r"'\''"))
    }
}
