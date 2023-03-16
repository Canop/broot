use {
    crate::{
        app::AppContext,
    },
    once_cell::sync::Lazy,
    std::path::Path,
    syntect::{
        easy::HighlightLines,
        parsing::SyntaxSet,
        highlighting::{Theme, ThemeSet},
    },
};

static SYNTAXES: &[u8] = include_bytes!("../../resources/syntect/syntaxes.bin");

pub static SYNTAXER: Lazy<Syntaxer> = Lazy::new(Syntaxer::default);

/// wrap heavy to initialize syntect things
pub struct Syntaxer {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
}
impl Default for Syntaxer {
    fn default() -> Self {
        Self {
            syntax_set: time!(Debug, syntect::dumps::from_binary(SYNTAXES)),
            theme_set: ThemeSet::load_defaults(),
        }
    }
}

impl Syntaxer {
    pub fn available_themes(
        &self
    ) -> std::collections::btree_map::Keys<String, Theme> {
        self.theme_set.themes.keys()
    }

    pub fn highlighter_for(
        &self,
        path: &Path,
        con: &AppContext,
    ) -> Option<HighlightLines<'_>> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.syntax_set.find_syntax_by_extension(ext))
            .map(|syntax| {
                let theme = con.syntax_theme.unwrap_or_default();
                let theme = self.theme_set.themes.get(theme.syntect_name())
                    .unwrap_or_else(|| self.theme_set.themes.iter().next().unwrap().1);
                HighlightLines::new(syntax, theme)
            })
    }
}
