
mod content_match;
mod content_search_result;
mod magic_numbers;
mod needle;


pub use {
    content_match::ContentMatch,
    content_search_result::ContentSearchResult,
    needle::Needle,
};

pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

