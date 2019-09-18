use std::fmt;

pub struct ElidedString<'a> {
    pub parts: Vec<&'a str>,
}

/// we won't cut into smaller tokens
const CUTTABLE_TOKEN_THRESHOLD: usize = 7;

/// An ElidedString is the list of parts we keep after we removed
/// enough to fit a given width.
///
/// We try to remove parts from the longest tokens (parts without spaces).
///
/// Note: our heuristic works for broot status lines but isn't
/// expected to be a universally good solution.
impl<'a> ElidedString<'_> {
    /// return a vector of strings, whose total length, if you add an
    /// ellipsis between parts, fits the given width.
    pub fn from(text: &'a str, width: usize) -> ElidedString<'_> {
        let mut parts = Vec::new();
        if text.len() <= width {
            parts.push(text);
        } else {
            let mut to_remove = text.len() - width;
            let mut ranges = core_ranges(text);
            ranges.sort_unstable_by(|a, b| b.possible_gain().cmp(&a.possible_gain()));
            for i in 0..ranges.len() {
                if ranges[i].protected {
                    continue;
                }
                let gain = ranges[i].possible_gain().min(to_remove);
                to_remove -= gain;
                ranges[i].removed += gain;
                if to_remove == 0 {
                    break;
                }
            }
            if to_remove > 0 {
                // we failed the smart thing. Do the dumb one
                parts.push(&text[0..width]);
            } else {
                // it works! let's just make the parts
                let mut idx = 0;
                ranges.sort_unstable_by(|a, b| a.start.cmp(&b.start));
                for r in ranges {
                    if r.removed > 0 {
                        parts.push(&text[idx..r.removed_start()]);
                        idx = r.removed_end();
                    }
                }
                parts.push(&text[idx..]);
            }
        }
        ElidedString{ parts }
    }
    /// return the total length, assuming there's an ellipsis (of length 1)
    /// between parts.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.parts.len() + self.parts.iter().map(|s| s.len()).sum::<usize>() - 1
    }
}

impl fmt::Display for ElidedString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.parts.join("…"))
    }
}

#[derive(Debug)]
struct Range {
    start: usize,    // index of the first character making this range
    end: usize,      // index of the last character making this range
    removed: usize,  //
    protected: bool, // a protected range can't be cut
}
impl Range {
    fn new(start: usize, end: usize, protected: bool) -> Self {
        Self {
            start,
            end,
            removed: 0,
            protected: protected || (end - start + 1 < CUTTABLE_TOKEN_THRESHOLD),
        }
    }
    /// the number of chars which would be gained by removing
    ///  all chars and putting a one char ellipsis in place
    fn possible_gain(&self) -> usize {
        self.end - self.start - self.removed
    }
    fn removed_start(&self) -> usize {
        let d = self.end - self.start - self.removed;
        self.start + d / 2
    }
    fn removed_end(&self) -> usize {
        let d = self.end - self.start - self.removed;
        self.end - d + d / 2 + 1
    }
}

fn core_ranges(text: &str) -> Vec<Range> {
    let mut ranges = Vec::new();
    let mut start: Option<usize> = None; // start of non whitespace area
    let mut protected = false;
    for (idx, c) in text.chars().enumerate() {
        if c.is_ascii_whitespace() {
            if let Some(start) = start {
                if idx > start + 3 {
                    ranges.push(Range::new(
                        start + 1,
                        idx - 2,
                        protected,
                    ));
                }
            }
            start = None;
        } else if start.is_none() {
            // right now we protect tokens starting with a '<', mostly
            // to avoid cutting key sequences like "<alt><enter>"
            protected = c == '<';
            start = Some(idx);
        }
    }
    if let Some(start) = start {
        if text.len() > start + 3 {
            ranges.push(Range::new(
                start + 1,
                text.len() - 2,
                protected,
            ));
        }
    }
    ranges
}

//#[cfg(test)]
mod status_fitting_tests {

    use crate::elision::ElidedString;

    #[allow(dead_code)]
    fn check_width(raw: &str, width: usize) {
        let cut = ElidedString::from(raw, width);
        println!("{}", raw);
        println!("{}", cut);
        if raw.len() <= width {
            assert_eq!(cut.parts.len(), 1);
            assert_eq!(cut.parts[0], raw);
        } else {
            assert_eq!(cut.len(), width);
        }
        assert!(cut.len() <= width);
    }

    // warning: we just check the lengths are OK, we don't check the cut is raisonnable
    #[test]
    fn check_fitting_status() {
        check_width("bla bla bla bla", 10);
        check_width("Hit <enter> to mv : /bin/mv /home/dys/dev/broot/img/20181215-only-folders-with-size.png /home/dys/dev/toto.png", 40);
        check_width("Hit <enter> to mv : /bin/mv /home/dys/dev/broot/img/20181215-only-folders-with-size.png /home/dys/dev/toto.png", 80);
        check_width("Hit <enter> to mv : /bin/mv /home/dys/dev/broot/img/20181215-only-folders-with-size.png /home/dys/dev/toto.png", 20);
    }
}

