use {
    unicode_width::UnicodeWidthChar,
};

// return the counts in bytes and columns of the longest substring
// fitting the given number of columns
pub fn count_fitting(s: &str, columns_max: usize) -> (usize, usize) {
    let mut count_bytes = 0;
    let mut str_width = 0;
    for (idx, c) in s.char_indices() {
        let char_width = UnicodeWidthChar::width(c).unwrap_or(0);
        let next_str_width = str_width + char_width;
        if next_str_width > columns_max {
            break;
        }
        str_width = next_str_width;
        count_bytes = idx + c.len_utf8();
    }
    (count_bytes, str_width)
}

#[cfg(test)]
mod fitting_count_tests {
    use super::*;

    #[test]
    fn test_count_fitting() {
        assert_eq!(count_fitting("test", 3), (3, 3));
        assert_eq!(count_fitting("test", 5), (4, 4));
        let c12 = "Comunicações"; // normalized string (12 characters, 14 bytes)
        assert_eq!(c12.len(), 14);
        assert_eq!(c12.chars().count(), 12);
        assert_eq!(count_fitting(c12, 12), (14, 12));
        assert_eq!(count_fitting(c12, 10), (12, 10));
        assert_eq!(count_fitting(c12, 11), (13, 11));
        let c14 = "Comunicações"; // unnormalized string (14 characters, 16 bytes)
        assert_eq!(c14.len(), 16);
        assert_eq!(c14.chars().count(), 14);
        assert_eq!(count_fitting(c14, 12), (16, 12));
        let ja = "概要"; // each char takes 3 bytes and 2 columns
        assert_eq!(ja.len(), 6);
        assert_eq!(ja.chars().count(), 2);
        assert_eq!(count_fitting(ja, 1), (0, 0));
        assert_eq!(count_fitting(ja, 2), (3, 2));
        assert_eq!(count_fitting(ja, 3), (3, 2));
        assert_eq!(count_fitting(ja, 4), (6, 4));
        assert_eq!(count_fitting(ja, 5), (6, 4));
    }
}
