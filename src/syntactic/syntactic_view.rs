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
        io::{self, BufRead},
        path::{Path, PathBuf},
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

/// number of lines initially parsed (and shown before scroll).
/// It's best to have this greater than the screen height to
/// avoid two initial parsings.
const INITIAL_HEIGHT: usize = 150;

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
    path: PathBuf,
    lines: Vec<SyntacticLine>,
    content_height: usize, // bigger than lines.len() if not fully loaded
    scroll: i32,
    page_height: i32,
}

impl SyntacticView {
    /// try to load and parse the content or part of it.
    /// Check the correct encoding of the whole file
    ///  even when `full` is false.
    fn prepare(&mut self, full: bool) -> io::Result<()> {
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
                SYNTAXER.theme_set.themes.iter().next().unwrap().1
            }
        };
        let syntax_set = &SYNTAXER.syntax_set;
        let mut highlighter = HighlightFile::new(&self.path, syntax_set, theme)?;
        self.lines.clear();
        self.content_height = 0;
        for line in highlighter.reader.lines() {
            let line = line?;
            self.content_height += 1;
            if full || self.content_height < INITIAL_HEIGHT {
                let contents = highlighter.highlight_lines
                    .highlight(&line, &syntax_set)
                    .iter()
                    .map(|r| SyntacticRegion::from_syntect(r))
                    .collect();
                self.lines.push(SyntacticLine { contents });
            }
        }
        Ok(())
    }

    pub fn new(
        path: &Path,
    ) -> io::Result<Self> {
        let mut sv = Self {
            path: path.to_path_buf(),
            lines: Vec::new(),
            content_height: 0,
            scroll: 0,
            page_height: 0,
        };
        sv.prepare(false)?;
        Ok(sv)
    }

    pub fn is_fully_loaded(&self) -> bool {
        self.content_height == self.lines.len()
    }

    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        let old_scroll = self.scroll;
        self.scroll = (self.scroll + cmd.to_lines(self.page_height))
            .min(self.content_height as i32 - self.page_height + 1)
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
        self.page_height = area.height as i32;
        if !self.is_fully_loaded() && self.scroll + self.page_height > self.lines.len() as i32 {
            debug!("now fully loading");
            self.prepare(true)?;
        }
        let line_count = area.height as usize;
        let styles = &panel_skin.styles;
        let bg: Option<Color> = styles.preview.get_bg();
        if bg.is_none() {
            w.queue(ResetColor)?;
        }
        // code is thiner than the area:
        // * 1 char margin at left
        // * space for the scrollbar at right
        let code_width = area.width as usize - 2;
        let scrollbar = area.scrollbar(self.scroll, self.content_height as i32);
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

