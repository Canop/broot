use phf::{
    Set,
    phf_set,
};

/// a short list of extensions that shouldn't be searched
///  by content
///
/// Entries must be lowercase: lookups are done case-insensitively
/// (see `is_known_binary`).
///
/// If you feel this list should maybe be changed, contact
/// me on miaou or raise an issue.
static BINARY_EXTENSIONS: Set<&'static str> = phf_set! {
    "a",
    "aif",
    "ap_",
    "apk",
    "bin",
    "bmp",
    "bzip",
    "bzip2",
    "cab",
    "class",
    "com",
    "crx",
    "dat",
    "db",
    "dbf",
    "deb",
    "doc",
    "docx",
    "eps",
    "exe",
    "dll",
    "gif",
    "gz",
    "gzip",
    "ico",
    "iso",
    "jar",
    "jpg",
    "jpeg",
    "lz4",
    "mdb",
    "mp3",
    "mp4",
    "mpa",
    "mpg",
    "mpeg",
    "msi",
    "o",
    "odf",
    "odp",
    "ods",
    "odt",
    "ogg",
    "pdb",
    "pdf",
    "pkg",
    "png",
    "ppt",
    "pptx",
    "psd",
    "ps",
    "rar",
    "rpm",
    "rsrc",
    "rtf",
    "so",
    "tar",
    "tar.gz",
    "ttf",
    "tgz",
    "xls",
    "xlsx",
    "vob",
    "vsd",
    "vsdx",
    "war",
    "wasm",
    "wav",
    "woff",
    "woff2",
    "zip",
    "z",
};

/// tells whether the file extension is one of a file format
/// which shouldn't be searched as text
///
/// The comparison is case-insensitive.
#[must_use]
pub fn is_known_binary(ext: &str) -> bool {
    BINARY_EXTENSIONS.contains(ext.to_ascii_lowercase().as_str())
}
