use {
    phf::{
        Set,
        phf_set,
    },
    std::{
        fs::File,
        io::{
            self,
            Read,
        },
        path::Path,
    },
};

pub const MIN_FILE_SIZE: usize = 100;

// bzip2 is checked on 3 bytes because the fourth one is the block size digit
static SIGNATURES_3: Set<[u8; 3]> = phf_set! {
    [ 0x42u8, 0x5A, 0x68 ], // bzip2
};

// signatures starting with 00, FF or FE don't need to be put here
// note: the phf_set macro doesn't seem to allow u32 literals like 0x504B0304
static SIGNATURES_4: Set<[u8; 4]> = phf_set! {
    [ 0x50u8, 0x4B, 0x03, 0x04 ], // zip file format and formats based on it, such as EPUB, JAR, ODF, OOXML
    [ 0x50, 0x4B, 0x05, 0x06 ], // zip file format and formats based on it, such as EPUB, JAR, ODF, OOXML
    [ 0x50, 0x4B, 0x07, 0x08 ], // zip file format and formats based on it, such as EPUB, JAR, ODF, OOXML
    [ 0xED, 0xAB, 0xEE, 0xDB ], // rpm
    [ 0x49, 0x49, 0x2A, 0x00 ], // tif
    [ 0x4D, 0x4D, 0x00, 0x2A ], // tiff
    [ 0x7F, 0x45, 0x4C, 0x46 ], // elf
    [ 0xCA, 0xFE, 0xBA, 0xBE ], // java class
    [ 0x25, 0x21, 0x50, 0x53 ], // ps
    [ 0x4F, 0x67, 0x67, 0x53 ], // ogg
    [ 0x38, 0x42, 0x50, 0x53 ], // psd
    [ 0x57, 0x41, 0x56, 0x45 ], // wave
    [ 0x41, 0x56, 0x49, 0x20 ], // avi
    [ 0x4D, 0x54, 0x68, 0x64 ], // midi
    [ 0xD0, 0xCF, 0x11, 0xE0 ], // old MS Office things
    [ 0x43, 0x72, 0x32, 0x34 ], // old Chrome extensions
    [ 0x78, 0x61, 0x72, 0x21 ], // xar
    [ 0x75, 0x73, 0x74, 0x61 ], // tar
    [ 0x37, 0x7A, 0xBC, 0xAF ], // 7zip
    [ 0x4D, 0x53, 0x43, 0x46 ], // Microsoft Cabinet file
    [ 0x52, 0x49, 0x46, 0x46 ], // riff (including WebP)
    [ 0x47, 0x49, 0x46, 0x38 ], // gif (common start of GIF87a and GIF89a )
    [ 0x4C, 0x5A, 0x49, 0x50 ], // lzip
    [ 0xCE, 0xFA, 0xED, 0xFE ], // Mach-O
    [ 0xCF, 0xFA, 0xED, 0xFE ], // Mach-O
    [ 0x46, 0x4C, 0x49, 0x46 ], // flif
    [ 0x62, 0x76, 0x78, 0x32 ], // lzfse
    [ 0xFD, 0x37, 0x7A, 0x58 ], // xz
    [ 0x28, 0xB5, 0x2F, 0xFD ], // zstd
};

/// return true when the first bytes of the file aren't polite or match one
/// of the known binary signatures.
/// Signatures are taken in <https://en.wikipedia.org/wiki/List_of_file_signatures>
/// Some signatures are omitted from list because they would not go past the
/// specific test of the first byte anyway.
///
/// If you feel this list should maybe be changed, contact
/// me on miaou or raise an issue.
#[must_use]
pub fn is_known_binary(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }
    let c = bytes[0];
    if c < 9 || (c > 13 && c < 32) || c >= 254 {
        // c < 9 include several signatures
        // 14 to 31 includes several signatures among them some variants of zip, gzip, etc.
        // FE is "þ", FF is "ÿ"
        // the FE/FF cases includes several signatures like Mach-O, jpeg or mpeg
        // TODO Some non ASCII UTF-8 chars start with FE or FF - check it's OK
        return true;
    }
    if SIGNATURES_3.contains(&bytes[0..3]) {
        return true;
    }
    if SIGNATURES_4.contains(&bytes[0..4]) {
        return true;
    }
    false
}

#[test]
fn test_compressed_magic_numbers() {
    assert!(is_known_binary(&[0x1F, 0x8B, 0x08, 0x00])); // gzip
    assert!(is_known_binary(&[0xFD, 0x37, 0x7A, 0x58])); // xz
    assert!(is_known_binary(&[0x28, 0xB5, 0x2F, 0xFD])); // zstd
    for digit in b'1'..=b'9' {
        assert!(is_known_binary(&[0x42, 0x5A, 0x68, digit])); // bzip2
    }
    assert!(!is_known_binary(b"Sep 20 12:00:00 host proc: ok"));
    assert!(!is_known_binary(b"BZip2 is a compression format"));
}

/// Tell whether the file i
pub fn is_file_known_binary<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    let mut buf = [0; 4];
    let mut file = File::open(path)?;
    let n = file.read(&mut buf)?;
    Ok(is_known_binary(&buf[0..n]))
}
