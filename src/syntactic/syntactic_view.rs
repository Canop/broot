
use {
    super::Syntaxer,
    crate::{
        command::{ScrollCommand},
        display::{CropWriter, LONG_SPACE, Screen, W},
        errors::ProgramError,
        skin::PanelSkin,
    },
    crossterm::{
        cursor,
        style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
        QueueableCommand,
    },
    std::{
        fs::File,
        io::{self, BufRead},
        path::Path,
    },
    syntect::{
        highlighting::Style,
        easy::HighlightFile,
    },
    termimad::Area,
};

#[derive(Debug)]
pub struct SyntacticRegion {
    pub fg: Color,
    pub string: String,
}

impl SyntacticRegion {
    pub fn from_syntect(region: &(Style, &str)) -> Self {
        let fg = Color::Rgb {
            r: region.0.foreground.r,
            g: region.0.foreground.g,
            b: region.0.foreground.b,
        };
        let string = str::replace(region.1, '\t', "    ");
        Self { fg, string }
    }
}

pub struct SyntacticLine {
    pub contents: Vec<SyntacticRegion>,
}

pub struct SyntacticView {
    lines: Vec<SyntacticLine>,
    scroll: i32,
    page_height: i32,
}

impl SyntacticView {
    pub fn new(
        path: &Path,
    ) -> io::Result<Option<Self>> {
        lazy_static! {
            static ref SYNTAXER: Syntaxer = Syntaxer::new();
        }
        //let theme_key = "base16-ocean.dark";
        //let theme_key = "Solarized (dark)";
        //let theme_key = "base16-eighties.dark";
        let theme_key = "base16-mocha.dark";
        let theme = match SYNTAXER.theme_set.themes.get(theme_key) {
            Some(theme) => theme,
            None => {
                warn!("theme not found : {:?}", theme_key);
                return Ok(None);
            }
        };
        let syntax_set = &SYNTAXER.syntax_set;
        let mut highlighter = match HighlightFile::new(path, syntax_set, theme) {
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
                .map(|r| SyntacticRegion::from_syntect(r))
                .collect();
            lines.push(SyntacticLine { contents });
        }
        Ok(Some(Self {
            lines,
            scroll: 0,
            page_height: 0,
        }))
    }

    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        let old_scroll = self.scroll;
        self.scroll = (self.scroll + cmd.to_lines(self.page_height))
            .min(self.lines.len() as i32 - self.page_height + 1)
            .max(0);
        self.scroll != old_scroll
    }

    pub fn display(
        &mut self,
        w: &mut W,
        _screen: &Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        let line_count = area.height as usize;
        self.page_height = area.height as i32;
        let styles = &panel_skin.styles;
        let bg: Option<Color> = styles.preview.get_bg();
        if bg.is_none() {
            w.queue(ResetColor)?;
        }
        // code is thiner than the area:
        // * 1 char margin at left
        // * space for the scrollbar at right
        let code_width = area.width as usize - 2;
        let scrollbar = area.scrollbar(self.scroll, self.lines.len() as i32);
        let scrollbar_fg = styles.scrollbar_thumb.get_fg()
            .or(styles.preview.get_fg())
            .unwrap_or_else(|| Color::White);
        for y in 0..line_count {
            w.queue(cursor::MoveTo(area.left, y as u16 + area.top))?;
            if let Some(bg) = bg {
                w.queue(SetBackgroundColor(bg))?;
            }
            w.queue(Print(' '))?;
            let mut cw = CropWriter::new(w, code_width);
            let cw = &mut cw;
            if let Some(line) = self.lines.get(self.scroll as usize + y) {
                for content in &line.contents {
                    cw.w.queue(SetForegroundColor(content.fg))?;
                    cw.queue_unstyled_str(&content.string)?;
                }
            }
            cw.fill(&styles.preview, LONG_SPACE)?;
            if let Some(bg) = bg {
                // this should not be needed, so there's a
                // bug somewhere
                w.queue(SetBackgroundColor(bg))?;
            }
            if is_thumb(y, scrollbar) {
                w.queue(SetForegroundColor(scrollbar_fg))?;
                w.queue(Print('▐'))?;
            } else {
                w.queue(Print(' '))?;
            }
        }
        Ok(())
    }
}

fn is_thumb(y: usize, scrollbar: Option<(u16, u16)>) -> bool {
    if let Some((sctop, scbottom)) = scrollbar {
        let y = y as u16;
        if sctop <= y && y <= scbottom {
            return true;
        }
    }
    false
}

