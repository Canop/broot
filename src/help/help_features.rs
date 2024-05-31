

/// find the list of optional features which are enabled
pub fn list() -> Vec<(&'static str, &'static str)> {
    #[allow(unused_mut)]
    let mut features: Vec<(&'static str, &'static str)> = Vec::new();

    #[cfg(not(any(target_family = "windows", target_os = "android")))]
    features.push(("permissions", "allow showing file mode, owner and group"));

    #[cfg(feature = "clipboard")]
    features.push((
        "clipboard",
        ":copy_path (copying the current path), and :input_paste (pasting into the input)",
    ));

    #[cfg(feature = "trash")]
    features.push((
        "trash",
        ":trash, :open_trash, :restore_trashed_file, :purge_trash, :delete_trashed_file",
    ));

    features
}
