use regex::Regex;

#[derive(Clone, Debug)]
pub struct VerbInvocation {
    pub name: String,
    pub args: Option<String>,
}
impl VerbInvocation {
    pub fn from(invocation: &str) -> VerbInvocation {
        let caps = regex!(r"^(\S*)\s*(.+?)?\s*$").captures(invocation).unwrap();
        let name = caps.get(1).unwrap().as_str().to_string();
        let args = caps.get(2).map(|c| c.as_str().to_string());
        VerbInvocation { name, args }
    }
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }
    pub fn to_string_for_name(&self, name: &str) -> String {
        match self.args {
            Some(ref args) => format!("{} {}", name, args),
            None => name.to_owned(),
        }
    }
}
