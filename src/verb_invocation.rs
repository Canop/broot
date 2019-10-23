use regex::Regex;

#[derive(Clone, Debug)]
pub struct VerbInvocation {
    pub key: String, // this "key" name starts to be confusing... any idea?
    pub args: Option<String>,
}
impl VerbInvocation {
    pub fn from(invocation: &str) -> VerbInvocation {
        let caps = regex!(r"^(\S*)\s*(.+?)?\s*$").captures(invocation).unwrap();
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
