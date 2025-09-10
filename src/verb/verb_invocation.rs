use std::fmt;

/// the verb and its arguments, making the invocation.
/// When coming from parsing, the args is Some as soon
/// as there's a separator (i.e. it's "" in "cp ")
#[derive(Clone, Debug, PartialEq)]
pub struct VerbInvocation {
    pub name: String,
    pub args: Option<String>,
    pub bang: bool,
}

impl fmt::Display for VerbInvocation {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, ":")?;
        if self.bang {
            write!(f, "!")?;
        }
        write!(f, "{}", &self.name)?;
        if let Some(args) = &self.args {
            write!(f, " {}", &args)?;
        }
        Ok(())
    }
}

impl VerbInvocation {
    pub fn new<T: Into<String>>(
        name: T,
        args: Option<T>,
        bang: bool,
    ) -> Self {
        Self {
            name: name.into(),
            args: args.map(|s| s.into()),
            bang,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }
    /// build a new String
    pub fn complete_name(&self) -> String {
        if self.bang {
            format!("{}_tab", &self.name)
        } else {
            self.name.clone()
        }
    }
    /// basically return the invocation but allow another name (the shortcut
    /// or a variant)
    pub fn to_string_for_name(
        &self,
        name: &str,
    ) -> String {
        let mut s = String::new();
        if self.bang {
            s.push('!');
        }
        s.push_str(name);
        if let Some(args) = &self.args {
            s.push(' ');
            s.push_str(args);
        }
        s
    }
}

impl From<&str> for VerbInvocation {
    /// Parse a string being or describing the invocation of a verb with its
    /// arguments and optional bang. The leading space or colon must
    /// have been stripped before.
    ///
    /// Examples:
    ///  "mv"       -> name: "mv"
    ///  "!mv"      -> name: "mv", bang
    ///  "mv a b"   -> name: "mv", args: "a b"
    ///  "mv!a b"   -> name: "mv", args: "a b", bang
    ///  "a-b  c"   -> name: "a-b", args: "c", bang
    ///  "-sp"      -> name: "-", args: "sp"
    ///  "-a b"     -> name: "-", args: "a b"
    ///  "-a b"     -> name: "-", args: "a b"
    ///  "--a"      -> name: "--", args: "a"
    ///
    /// Notes:
    /// 1. A name is either "special" (only made of non alpha characters)
    ///    or normal (starting with an alpha character). Special names don't
    ///    need a space afterwards, as the first alpha character will start
    ///    the args.
    /// 2. The space or colon after the name is optional if there's a bang
    ///    after the name: the bang is the separator.
    /// 3. Duplicate separators before args are ignored (they're usually typos)
    /// 4. An opening parenthesis starts args
    fn from(invocation: &str) -> Self {
        let mut bang_before = false;
        let mut name = String::new();
        let mut bang_after = false;
        let mut args: Option<String> = None;
        let mut name_is_special = false;
        for c in invocation.chars() {
            if let Some(args) = args.as_mut() {
                if args.is_empty() && (c == ' ' || c == ':') {
                    // we don't want args starting with a space just because
                    // they're doubled or are optional after a special name
                } else {
                    args.push(c);
                }
                continue;
            }
            if c == ' ' || c == ':' {
                args = Some(String::new());
                continue;
            }
            if c == '(' {
                args = Some(c.to_string());
                continue;
            }
            if c == '!' {
                if !name.is_empty() {
                    bang_after = true;
                    args = Some(String::new());
                } else {
                    bang_before = true;
                }
                continue;
            }
            if name.is_empty() {
                name.push(c);
                if !c.is_alphabetic() {
                    name_is_special = true;
                }
                continue;
            }
            if c.is_alphabetic() && name_is_special {
                // this isn't part of the name anymore, it's part of the args
                args = Some(c.to_string());
                continue;
            }
            name.push(c);
        }
        let bang = bang_before || bang_after;
        VerbInvocation {
            name,
            args,
            bang,
        }
    }
}

#[cfg(test)]
mod verb_invocation_tests {
    use super::*;

    #[test]
    fn check_special_chars() {
        assert_eq!(
            VerbInvocation::from("-sdp"),
            VerbInvocation::new("-", Some("sdp"), false),
        );
        assert_eq!(
            VerbInvocation::from("!-sdp"),
            VerbInvocation::new("-", Some("sdp"), true),
        );
        assert_eq!(
            VerbInvocation::from("-!sdp"),
            VerbInvocation::new("-", Some("sdp"), true),
        );
        assert_eq!(
            VerbInvocation::from("-! sdp"),
            VerbInvocation::new("-", Some("sdp"), true),
        );
        assert_eq!(
            VerbInvocation::from("!@a b"),
            VerbInvocation::new("@", Some("a b"), true),
        );
        assert_eq!(
            VerbInvocation::from("!@%a b"),
            VerbInvocation::new("@%", Some("a b"), true),
        );
        assert_eq!(
            VerbInvocation::from("22a b"),
            VerbInvocation::new("22", Some("a b"), false),
        );
        assert_eq!(
            VerbInvocation::from("22!a b"),
            VerbInvocation::new("22", Some("a b"), true),
        );
        assert_eq!(
            VerbInvocation::from("22 !a b"),
            VerbInvocation::new("22", Some("!a b"), false),
        );
        assert_eq!(
            VerbInvocation::from("a$b4!r"),
            VerbInvocation::new("a$b4", Some("r"), true),
        );
        assert_eq!(
            VerbInvocation::from("a-b c"),
            VerbInvocation::new("a-b", Some("c"), false),
        );
    }

    #[test]
    fn check_verb_invocation_parsing_empty_arg() {
        // those tests focus mainly on the distinction between
        // None and Some("") for the args, distinction which matters
        // for inline help
        assert_eq!(VerbInvocation::from("!mv"), VerbInvocation::new("mv", None, true),);
        assert_eq!(
            VerbInvocation::from("mva!"),
            VerbInvocation::new("mva", Some(""), true),
        );
        assert_eq!(
            VerbInvocation::from("cp "),
            VerbInvocation::new("cp", Some(""), false),
        );
        assert_eq!(
            VerbInvocation::from("cp ../"),
            VerbInvocation::new("cp", Some("../"), false),
        );
    }

    #[test]
    fn check_verb_invocation_parsing_post_bang() {
        // ignoring post_bang (see issue #326)
        assert_eq!(
            VerbInvocation::from("mva!a"),
            VerbInvocation::new("mva", Some("a"), true),
        );
        assert_eq!(VerbInvocation::from("!!!"), VerbInvocation::new("", None, true),);
    }

    #[test]
    fn check_verb_invocation_parsing_empty_verb() {
        // there's currently no meaning for the empty verb, it's "reserved"
        // and will probably not be used as it may need a distinction between
        // one and two initial spaces in the input
        assert_eq!(VerbInvocation::from(""), VerbInvocation::new("", None, false),);
        assert_eq!(VerbInvocation::from("!"), VerbInvocation::new("", None, true),);
        assert_eq!(VerbInvocation::from("!! "), VerbInvocation::new("", Some(""), true),);
        assert_eq!(
            VerbInvocation::from("!! a"),
            VerbInvocation::new("", Some("a"), true),
        );
    }

    #[test]
    fn check_verb_invocation_parsing_oddities() {
        // checking some corner cases
        assert_eq!(
            VerbInvocation::from("!!a"), // the second bang is ignored
            VerbInvocation::new("a", None, true),
        );
        assert_eq!(
            VerbInvocation::from("!!"), // the second bang is ignored
            VerbInvocation::new("", None, true),
        );
        assert_eq!(
            VerbInvocation::from("a ! !"),
            VerbInvocation::new("a", Some("! !"), false),
        );
        assert_eq!(
            VerbInvocation::from("!a !a"),
            VerbInvocation::new("a", Some("!a"), true),
        );
        assert_eq!(
            VerbInvocation::from("a! ! //"),
            VerbInvocation::new("a", Some("! //"), true),
        );
        assert_eq!(
            VerbInvocation::from(".. .."),
            VerbInvocation::new("..", Some(".."), false),
        );
    }
}
