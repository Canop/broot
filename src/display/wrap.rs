//! Line wrapping of previews: hard wrap at cell boundaries.

use unicode_width::UnicodeWidthChar;

/// Number of cells a tab takes in previews
pub const TAB_WIDTH: usize = 2;

/// Width in cells of a char displayed in a preview: tabs are expanded,
/// control chars (including end of line ones) take no room.
pub fn char_width(c: char) -> usize {
    if c == '\t' {
        TAB_WIDTH
    } else {
        c.width().unwrap_or(0)
    }
}

/// Return the indexes, in chars, at which start the rows after the
/// first one when the text is wrapped at `width` cells.
///
/// A wide char never straddles two rows (unless wider than the row).
/// Zero width chars stay on the row of the previous char.
pub fn row_starts(
    chars: impl Iterator<Item = char>,
    width: usize,
) -> Vec<usize> {
    let width = width.max(1);
    let mut starts = Vec::new();
    let mut x = 0;
    for (i, c) in chars.enumerate() {
        let w = char_width(c);
        if w > 0 && x > 0 && x + w > width {
            starts.push(i);
            x = 0;
        }
        x += w;
    }
    starts
}

/// Return the number of rows the text takes when wrapped at `width` cells
/// (at least 1)
pub fn row_count(
    chars: impl Iterator<Item = char>,
    width: usize,
) -> usize {
    let width = width.max(1);
    let mut count = 1;
    let mut x = 0;
    for c in chars {
        let w = char_width(c);
        if w > 0 && x > 0 && x + w > width {
            count += 1;
            x = 0;
        }
        x += w;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii() {
        assert_eq!(row_starts("".chars(), 5), Vec::<usize>::new());
        assert_eq!(row_starts("abcde".chars(), 5), Vec::<usize>::new());
        assert_eq!(row_starts("abcdef".chars(), 5), vec![5]);
        assert_eq!(row_starts("abcdefghijk".chars(), 5), vec![5, 10]);
        assert_eq!(row_count("abcdefghijk".chars(), 5), 3);
        assert_eq!(row_count("".chars(), 5), 1);
    }

    #[test]
    fn end_of_line_takes_no_room() {
        assert_eq!(row_starts("abcde\n".chars(), 5), Vec::<usize>::new());
        assert_eq!(row_starts("abcde\r\n".chars(), 5), Vec::<usize>::new());
        assert_eq!(row_count("abcde\n".chars(), 5), 1);
    }

    #[test]
    fn tabs() {
        // a tab takes TAB_WIDTH cells
        assert_eq!(row_starts("\tab".chars(), 4), Vec::<usize>::new());
        assert_eq!(row_starts("\tabc".chars(), 4), vec![3]);
    }

    #[test]
    fn wide_chars() {
        // 日 and 本 are 2 cells wide: a wide char doesn't straddle rows
        assert_eq!(row_starts("a日本".chars(), 4), vec![2]);
        assert_eq!(row_starts("日本語".chars(), 4), vec![2]);
        assert_eq!(row_count("日本語".chars(), 4), 2);
    }

    #[test]
    fn combining_marks_stay_with_base() {
        // e + combining acute accent (zero width)
        let s = "abcde\u{301}f";
        assert_eq!(row_starts(s.chars(), 5), vec![6]);
    }

    #[test]
    fn tiny_width() {
        assert_eq!(row_starts("abc".chars(), 0), vec![1, 2]);
        assert_eq!(row_starts("日本".chars(), 1), vec![1]);
    }
}
