
use {
    memmap::Mmap,
};

/// This is an experimental or temporary implementation of Boyer-Moore.
/// DO NOT COPY IT: You'll easily find better source codes and crates
/// for example https://crates.io/crates/needle
/// In common cases, using Boyer-Moore brings no substantial
/// performance improvements over naive approachs, so I might
/// totally remove this code from broot.
#[derive(Clone)]
pub struct BoyerMoore {
    bad_characters_table: [usize; 256],
    good_suffixes_table: Vec<usize>,
}

impl BoyerMoore {
    pub fn new(needle: &[u8]) -> Self {
        Self {
            bad_characters_table: bad_characters_table(&needle),
            good_suffixes_table: good_suffixes_table(&needle),
        }
    }
    pub unsafe fn find(&self, mut pos: usize, hay: &Mmap, needle: &[u8]) -> Option<usize> {
        let max_pos = hay.len() - needle.len();
        while pos <= max_pos {
            let mut needle_pos = needle.len() - 1;
            while hay.get_unchecked(pos + needle_pos) == needle.get_unchecked(needle_pos) {
                if needle_pos == 0 {
                    return Some(pos);
                }
                needle_pos -= 1;
            }
            let bad_char = *hay.get_unchecked(pos + needle.len() - 1);
            pos += std::cmp::max(
                self.bad_characters_table.get_unchecked(bad_char as usize),
                self.good_suffixes_table.get_unchecked(needle_pos),
            );
        }
        None
    }
}

fn bad_characters_table(needle: &[u8]) -> [usize; 256] {
    let len = needle.len();
    let mut table = [len; 256];
    for i in 0..len-1 {
        table[needle[i] as usize] = len - i - 1;
    }
    table
}

fn get_suffix_table(needle: &[u8]) -> Vec<usize> {
    let len = needle.len();
    let mut table = vec![0; len];
    for suffix_len in 1 .. len {
        let mut found_suffix = false;
        for i in (0 .. len - suffix_len).rev() {
            if table[i + suffix_len - 1] == suffix_len - 1 && needle[i] == needle[len - suffix_len] {
                table[i + suffix_len - 1] = suffix_len;
                found_suffix = true;
            }
        }
        if !found_suffix {
            break;
        }
    }
    table
}

fn good_suffixes_table(needle: &[u8]) -> Vec<usize> {
    let suffixes = get_suffix_table(&needle);
    let len = needle.len();
    let mut table = vec![len - 1; len];
    for (i, suffix_len) in suffixes.into_iter().enumerate() {
        let needle_index = len - suffix_len - 1;
        let skip = len - i - 1;
        if table[needle_index] > skip {
            table[needle_index] = skip;
        }
    }
    table[len - 1] = 1;
    table
}

