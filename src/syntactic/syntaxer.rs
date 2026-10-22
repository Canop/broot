use {
    crate::app::AppContext,
    once_cell::sync::Lazy,
    std::path::Path,
    syntect::{
        easy::{
            HighlightLines,
            HighlightOptions,
        },
        highlighting::{
            Theme,
            ThemeSet,
        },
        parsing::SyntaxSet,
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
            syntax_set: time!(
                Debug,
                syntect::dumps::from_uncompressed_data(SYNTAXES).unwrap()
            ),
            theme_set: ThemeSet::load_defaults(),
        }
    }
}

impl Syntaxer {
    pub fn available_themes(&self) -> std::collections::btree_map::Keys<'_, String, Theme> {
        self.theme_set.themes.keys()
    }

    pub fn highlighter_for<'s>(
        &'s self,
        path: &Path,
        con: &'s AppContext,
    ) -> Option<HighlightLines<'s>> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| self.syntax_set.find_syntax_by_extension(ext))
            .map(|syntax| {
                let theme = match (con.syntax_theme, &con.custom_syntax_theme) {
                    (None, Some(theme)) => theme,
                    (theme, _) => self
                        .theme_set
                        .themes
                        .get(theme.unwrap_or_default().syntect_name())
                        .unwrap_or_else(|| self.theme_set.themes.iter().next().unwrap().1),
                };
                let options = HighlightOptions {
                    ignore_errors: true,
                };
                HighlightLines::new(syntax, theme, options)
            })
    }
}
