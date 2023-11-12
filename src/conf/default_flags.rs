use {
    crate::{
        cli::Args,
        errors::ConfError,
    },
    clap::Parser,
    lazy_regex::*,
};

/// parse the 'default_flags' parameter of a conf.
pub fn parse_default_flags(s: &str) -> Result<Args, ConfError> {
    let prefixed;
    let mut tokens: Vec<&str> = if regex_is_match!("^[a-zA-Z]+$", s) {
        // this covers the old syntax like `default_flags: gh`
        prefixed = format!("-{s}");
        vec![&prefixed]
    } else {
        splitty::split_unquoted_whitespace(s).collect()
    };
    tokens.insert(0, "broot");
    Args::try_parse_from(&tokens)
        .map_err(|_| ConfError::InvalidDefaultFlags {
            flags: s.to_string()
        })
}
