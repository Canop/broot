use {
    crate::{
        app::AppContext,
    },
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
impl Default for Syntaxer {
    fn default() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_nonewlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }
}

impl Syntaxer {
    pub fn highlighter_for<'s, 'p>(
        &'s self,
        path: &'p Path,
        con: &AppContext,
    ) -> Option<HighlightLines<'s>> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.syntax_set.find_syntax_by_extension(ext))
            .map(|syntax| {
                // some OK themes:
                //  "base16-ocean.dark"
                //  "Solarized (dark)"
                //  "base16-eighties.dark"
                //  "base16-mocha.dark"
                let theme = con.syntax_theme.as_ref()
                    .and_then(|key| self.theme_set.themes.get(key))
                    .or_else(|| self.theme_set.themes.get("base16-mocha.dark"))
                    .unwrap_or_else(|| self.theme_set.themes.iter().next().unwrap().1);
                HighlightLines::new(syntax, &theme)
            })
    }
}
