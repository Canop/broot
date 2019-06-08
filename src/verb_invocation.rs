use regex::Regex;

#[derive(Clone, Debug)]
pub struct VerbInvocation {
    pub key: String,
    pub args: Option<String>,
}
impl VerbInvocation {
    pub fn from(invocation: &str) -> VerbInvocation {
        lazy_static! {
            static ref PARTS: Regex = Regex::new(r"^(\S*)\s*(.+?)?\s*$").unwrap();
        }
        let caps = PARTS.captures(invocation).unwrap(); // this regex should always match
        let key = caps.get(1).unwrap().as_str().to_string();
        let args = caps.get(2).map(|c| c.as_str().to_string());
        VerbInvocation { key, args }
    }
    pub fn is_empty(&self) -> bool {
        self.key.is_empty()
    }
    pub fn to_string_for_key(&self, key: &str) -> String {
        match self.args {
            Some(ref args) => format!("{} {}", key, args),
            None => key.to_owned(),
        }
    }
}
