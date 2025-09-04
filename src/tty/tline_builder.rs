use super::*;

/// A builder consuming a string assumed to contain TTY sequences and building a TLine.
#[derive(Debug, Default)]
pub struct TLineBuilder {
    cur: Option<TString>,
    strings: Vec<TString>,
}
impl TLineBuilder {
    pub fn read(
        &mut self,
        s: &str,
    ) {
        let mut parser = vte::Parser::new();
        parser.advance(self, s.as_bytes());
    }
    pub fn build(mut self) -> TLine {
        self.take_tstring();
        TLine {
            strings: self.strings,
        }
    }
    fn take_tstring(&mut self) {
        if let Some(cur) = self.cur.take() {
            self.push_tstring(cur);
        }
    }
    fn push_tstring(
        &mut self,
        tstring: TString,
    ) {
        if let Some(last) = self.strings.last_mut() {
            if last.csi == tstring.csi {
                last.raw.push_str(&tstring.raw);
                return;
            }
        }
        self.strings.push(tstring);
    }
}
impl vte::Perform for TLineBuilder {
    fn print(
        &mut self,
        c: char,
    ) {
        self.cur.get_or_insert_with(TString::default).raw.push(c);
    }
    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        if params.len() == 1 && params.iter().next() == Some(&[0]) {
            self.take_tstring();
            return;
        }
        if let Some(cur) = self.cur.as_mut() {
            if cur.raw.is_empty() {
                cur.push_csi(params, action);
                return;
            }
        }
        self.take_tstring();
        let mut cur = TString::default();
        cur.push_csi(params, action);
        self.cur = Some(cur);
    }
    fn execute(
        &mut self,
        _byte: u8,
    ) {
    }
    fn hook(
        &mut self,
        _params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        _action: char,
    ) {
    }
    fn put(
        &mut self,
        _byte: u8,
    ) {
    }
    fn unhook(&mut self) {}
    fn osc_dispatch(
        &mut self,
        _params: &[&[u8]],
        _bell_terminated: bool,
    ) {
    }
    fn esc_dispatch(
        &mut self,
        _intermediates: &[u8],
        _ignore: bool,
        _byte: u8,
    ) {
    }
}
