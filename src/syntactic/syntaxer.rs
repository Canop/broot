use {
    std::path::Path,
    syntect::{
        easy::HighlightLines,
        parsing::SyntaxSet,
        highlighting::ThemeSet,
    },
};

/// wrap heavy to initialize syntect things
pub struct Syntaxer {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
}
impl Syntaxer {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_nonewlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }
    pub fn highlighter_for<'s, 'p>(
        &'s self,
        path: &'p Path,
    ) -> Option<HighlightLines<'s>> {
        path.extension()
            .and_then(|e|e.to_str())
            .and_then(|ext| self.syntax_set.find_syntax_by_extension(ext))
            .map(|syntax| {
                //let theme_key = "base16-ocean.dark";
                //let theme_key = "Solarized (dark)";
                //let theme_key = "base16-eighties.dark";
                let theme_key = "base16-mocha.dark";
                let theme = match self.theme_set.themes.get(theme_key) {
                    Some(theme) => theme,
                    None => {
                        warn!("theme not found : {:?}", theme_key);
                        self.theme_set.themes.iter().next().unwrap().1
                    }
                };
                HighlightLines::new(syntax, &theme)
            })
    }
}
