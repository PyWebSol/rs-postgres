use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::easy::HighlightLines;

pub struct SyntaxHighlighter {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn highlight(&self, code: &str) -> Vec<(Style, String)> {
        let syntax = self.syntax_set.find_syntax_by_extension("sql").unwrap();
        let mut h = HighlightLines::new(syntax, &self.theme_set.themes["base16-ocean.dark"]);
        let regions = h.highlight_line(code, &self.syntax_set).unwrap();
        regions.into_iter().map(|(style, text)| (style, text.to_string())).collect()
    }
}
