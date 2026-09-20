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
    "br",
    "bz2",
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
    "ttf",
    "tgz",
    "xls",
    "xlsx",
    "xz",
    "vob",
    "vsd",
    "vsdx",
    "war",
    "wasm",
    "wav",
    "woff",
    "woff2",
    "zip",
    "zst",
    "zstd",
    "z",
};

/// tells whether the file extension is one of a file format
/// which shouldn't be searched as text
#[must_use]
pub fn is_known_binary(ext: &str) -> bool {
    if BINARY_EXTENSIONS.contains(ext) {
        return true;
    }
    ext.bytes().any(|b| b.is_ascii_uppercase())
        && BINARY_EXTENSIONS.contains(ext.to_ascii_lowercase().as_str())
}

#[test]
fn test_compressed_extensions() {
    assert!(is_known_binary("br"));
    assert!(is_known_binary("bz2"));
    assert!(is_known_binary("xz"));
    assert!(is_known_binary("zst"));
    assert!(is_known_binary("zstd"));
    assert!(is_known_binary("BZ2"));
    assert!(!is_known_binary("log"));
    assert!(!is_known_binary("rs"));
}

/// Compound archive names are covered by their last extension,
/// which is what `Path::extension` gives to `is_known_binary`
#[test]
fn test_compound_archive_extensions() {
    for name in ["a.tar.gz", "a.tar.bz2", "a.tar.xz", "a.tar.zst"] {
        let ext = std::path::Path::new(name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap();
        assert!(is_known_binary(ext), "{name} should be binary");
    }
}
