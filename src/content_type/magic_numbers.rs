
use {
    phf::{phf_set, Set},
    std::{
        path::Path,
        fs::File,
        io::{self, Read},
    },
};

pub const MIN_FILE_SIZE: usize = 100;

// those ones are now removed because of the extension filtering
// static SIGNATURES_2: [[u8;2];2] = [
//     [ 0x4D, 0x5A ], // exe, dll
//     [ 0x42, 0x4D ], // BMP - Is that still necessary ?
// ];

// those ones are now removed because of the extension filtering
// static SIGNATURES_3: [[u8;3];2] = [
//     [ 0x49, 0x44, 0x33 ], // mp3
//     [ 0x77, 0x4F, 0x46 ],  // WOFF
// ];

// signatures starting with 00, FF or FE don't need to be put here
// note: the phf_set macro doesn't seem to allow u32 literals like 0x504B0304
static SIGNATURES_4: Set<[u8; 4]> = phf_set! {
    [ 0x50, 0x4B, 0x03, 0x04 ], // zip file format and formats based on it, such as EPUB, JAR, ODF, OOXML
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
};

// those ones are now removed because of the extension and size filterings
// static SIGNATURES_5: [[u8;5];2] = [
//     [ 0x25, 0x50, 0x44, 0x46, 0x2d ], // pdf
//     [ 0x43, 0x44, 0x30, 0x30, 0x31 ], // iso (cd/dvd)
// ];

// those ones are now removed because of the extension filterings
// static SIGNATURES_6: [[u8;6];4] = [
//     [ 0x52, 0x61, 0x72, 0x21, 0x1A, 0x07 ], // rar
//     [ 0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A ], // png
//     [ 0x21, 0x3C, 0x61, 0x72, 0x63, 0x68 ], // deb
//     [ 0x7B, 0x5C, 0x72, 0x74, 0x66, 0x31 ], // rtf
// ];


/// return true when the first bytes of the file aren't polite or match one
/// of the known binary signatures.
/// Signatures are taken in https://en.wikipedia.org/wiki/List_of_file_signatures
/// Some signatures are omitted from list because they would not go past the
/// specific test of the first byte anyway.
///
/// If you feel this list should maybe be changed, contact
/// me on miaou or raise an issue.
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
    // for signature in &SIGNATURES_2 {
    //     if signature == &bytes[0..2] {
    //         return true;
    //     }
    // }
    // for signature in &SIGNATURES_3 {
    //     if signature == &bytes[0..3] {
    //         return true;
    //     }
    // }
    if SIGNATURES_4.contains(&bytes[0..4]) {
        return true;
    }
    // for signature in &SIGNATURES_5 {
    //     if signature == &bytes[0..5] {
    //         return true;
    //     }
    // }
    // for signature in &SIGNATURES_6 {
    //     if signature == &bytes[0..6] {
    //         return true;
    //     }
    // }
    false
}

/// Tell whether the file i
pub fn is_file_known_binary<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    let mut buf = [0; 4];
    let mut file = File::open(path)?;
    let n = file.read(&mut buf)?;
    Ok(is_known_binary(&buf[0..n]))
}
