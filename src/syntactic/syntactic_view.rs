
use {
    crate::{
        display::{CropWriter, LONG_SPACE, Screen, W},
        errors::ProgramError,
        skin::PanelSkin,
    },
    crossterm::{
        cursor,
        style::{Color},
        QueueableCommand,
    },
    std::{
        fs::File,
        io::{self, BufRead},
        path::Path,
    },
    syntect::{
        parsing::SyntaxSet,
        highlighting::{ThemeSet, Style},
        easy::HighlightFile,
    },
    termimad::{Area, CompoundStyle},
};

pub struct SyntacticRegion {
    pub style: CompoundStyle,
    pub string: String,
}

pub struct SyntacticLine {
    pub contents: Vec<SyntacticRegion>,
}

pub struct SyntacticView {
    pub scroll: usize,
    pub lines: Vec<SyntacticLine>,
}

/// converts a syntect region to one easier to use with crossterm
fn convert_region(region: &(Style, &str)) -> SyntacticRegion {
    let style = CompoundStyle::with_fg(Color::Rgb {
            r: region.0.foreground.r,
            g: region.0.foreground.g,
            b: region.0.foreground.b,
        });
    let string = region.1.to_string();
    SyntacticRegion { style, string }
}

impl SyntacticView {
    pub fn new(path: &Path) -> io::Result<Option<Self>> {
        let syntax_set = SyntaxSet::load_defaults_nonewlines();
        let theme_set = ThemeSet::load_defaults();
        let theme = &theme_set.themes["base16-ocean.dark"];
        let mut highlighter = match HighlightFile::new(path, &syntax_set, theme) {
            Ok(h) => h,
            Err(e) => {
                warn!("failed to hightlight file {:?} : {:?}", path, e);
                return Ok(None);
            }
        };
        let mut lines = Vec::new();
        let reader = io::BufReader::new(File::open(path)?);
        for line in reader.lines() {
            let line = line?;
            let contents = highlighter.highlight_lines
                .highlight(&line, &syntax_set)
                .iter()
                .map(convert_region)
                .collect();
            lines.push(SyntacticLine { contents });
        }
        Ok(Some(Self {
            scroll: 0,
            lines,
        }))
    }
    pub fn display(
        &mut self,
        w: &mut W,
        _screen: &Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        let line_count = area.height as usize;
        let styles = &panel_skin.styles;
        for y in 0..line_count {
            w.queue(cursor::MoveTo(area.left, y as u16 + area.top))?;
            let mut cw = CropWriter::new(w, area.width as usize);
            let cw = &mut cw;
            if y < self.lines.len() {
                for content in &self.lines[y].contents {
                    cw.queue_str(&content.style, &content.string)?;
                }
            }
            cw.fill(&styles.default, LONG_SPACE)?;
        }
        Ok(())
    }
}
