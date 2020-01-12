use std::fmt::Write;

use nom::bytes::complete::take_till;
use nom::character::complete::multispace0;
use nom::combinator::rest;
use nom::sequence::separated_pair;
use nom::IResult;

#[derive(Clone, Debug)]
pub struct VerbInvocation {
    pub name: String,
    pub args: Option<String>,
}
impl VerbInvocation {
    pub fn from(invocation: &str) -> VerbInvocation {
        // An invocation is a name, followed by whitespace, followed optionally
        // by arguments. The name is allowed to be empty.
        let parse_invocation = separated_pair(
            // name
            take_till(|c: char| c.is_whitespace()),
            multispace0,
            // args
            rest,
        );

        let result: IResult<&str, (&str, &str), ()> = parse_invocation(invocation);

        match result {
            Ok(("", (name, ""))) => VerbInvocation {
                name: name.to_string(),
                args: None,
            },
            Ok(("", (name, args))) => VerbInvocation {
                name: name.to_string(),
                args: Some(args.to_string()),
            },

            // It shouldn't be possible for the parser to fail under any input,
            // nor for the parser to consume less that the whole input, so just
            // issue a generic panic
            result => panic!(
                "Unexpectedly failed to parse verb invocation. Parse result: {:?}",
                result
            ),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    pub fn to_string_for_name(&self, mut name: String) -> String {
        if let Some(args) = &self.args {
            // Unwrap is fine because write! to string is infallible
            write!(&mut name, " {}", args).unwrap();
        }
        name
    }
}

#[cfg(test)]
mod test_verb_invocation {
    use super::VerbInvocation;

    #[inline]
    fn check_invocation(invocation: &VerbInvocation, name: &str, args: Option<&str>) {
        assert_eq!(&invocation.name, name);
        assert_eq!(invocation.args.as_ref().map(|s| s.as_str()), args);
    }

    #[test]
    fn new_empty() {
        check_invocation(&VerbInvocation::from(""), "", None);
    }

    #[test]
    fn new_named() {
        check_invocation(&VerbInvocation::from(":name"), ":name", None);
    }

    #[test]
    fn new_trailing_space() {
        check_invocation(&VerbInvocation::from(":name  "), ":name", None);
    }

    #[test]
    fn new_arg() {
        check_invocation(&VerbInvocation::from(":name arg1"), ":name", Some("arg1"));
    }

    #[test]
    fn new_args() {
        check_invocation(
            &VerbInvocation::from(":name   arg1 arg2 arg3"),
            ":name",
            Some("arg1 arg2 arg3"),
        );
    }

    #[test]
    fn new_args_trailing_space() {
        check_invocation(&VerbInvocation::from(":name arg  "), ":name", Some("arg  "));
    }
}
