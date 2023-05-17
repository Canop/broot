use {
    super::byte::Byte,
    crate::{
        command::ScrollCommand,
        display::{Screen, W},
        errors::ProgramError,
        skin::PanelSkin,
    },
    crokey::crossterm::{
        cursor,
        style::{Color, Print, SetForegroundColor},
        QueueableCommand,
    },
    memmap2::Mmap,
    std::{
        fs::File,
        io,
        path::PathBuf,
    },
    termimad::{Area, CropWriter, SPACE_FILLING},
};


pub struct HexLine {
    pub bytes: Vec<u8>, // from 1 to 16 bytes
}

/// a preview showing the content of a file in hexa
pub struct HexView {
    path: PathBuf,
    len: usize,
    scroll: usize,
    page_height: usize,
}

impl HexView {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let len = path.metadata()?.len() as usize;
        Ok(Self {
            path,
            len,
            scroll: 0,
            page_height: 0,
        })
    }
    pub fn line_count(&self) -> usize {
        self.len / 16 + usize::from(self.len % 16 != 0)
    }
    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        let old_scroll = self.scroll;
        self.scroll = cmd.apply(self.scroll, self.line_count(), self.page_height);
        self.scroll != old_scroll
    }
    pub fn select_first(&mut self) {
        self.scroll = 0;
    }
    pub fn select_last(&mut self) {
        if self.page_height < self.line_count() {
            self.scroll = self.line_count() - self.page_height;
        }
    }
    pub fn get_page(
        &mut self,
        start_line_idx: usize,
        line_count: usize,
    ) -> io::Result<Vec<HexLine>> {
        let file = File::open(&self.path)?;
        let mut lines = Vec::new();
        if self.len == 0 {
            return Ok(lines);
        }
        let mmap = unsafe { Mmap::map(&file)? };
        let new_len = mmap.len();
        if new_len != self.len {
            warn!("previewed file len changed from {} to {}", self.len, new_len);
            self.len = new_len;
        }
        let mut start_idx = 16 * start_line_idx;
        while start_idx < self.len {
            let line_len = 16.min(self.len - start_idx);
            let mut bytes: Vec<u8> = vec![0; line_len];
            bytes[0..line_len].copy_from_slice(&mmap[start_idx..start_idx + line_len]);
            lines.push(HexLine { bytes });
            if lines.len() >= line_count {
                break;
            }
            start_idx += line_len;
        }
        Ok(lines)
    }
    pub fn display(
        &mut self,
        w: &mut W,
        _screen: Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        let line_count = area.height as usize;
        self.page_height = area.height as usize;
        let page = self.get_page(self.scroll, line_count)?;
        let addresses_len = if self.len < 0xffff {
            4
        } else if self.len < 0xff_ffff {
            6
        } else {
            8
        };
        let mut margin_around_addresses = false;
        let styles = &panel_skin.styles;
        let mut left_margin = false;
        let mut addresses = false;
        let mut hex_middle_space = false;
        let mut chars_middle_space = false;
        let mut inter_hex = false;
        let mut chars = false;
        const MIN: i32 =
            1    // margin
            + 32 // 32 hex
            + 1; // scrollbar
        let mut rem = area.width as i32 - MIN;
        if rem > 17 {
            chars = true;
            rem -= 17;
        }
        if rem > 16 {
            inter_hex = true;
            rem -= 16;
        }
        if rem > addresses_len {
            addresses = true;
            rem -= addresses_len;
        }
        if rem > 1 {
            hex_middle_space = true;
            rem -= 1;
        }
        if rem > 1 {
            left_margin = true;
            rem -= 1;
        }
        if rem > 1 {
            chars_middle_space = true;
            rem -= 1;
        }
        if addresses && rem >= 2 {
            margin_around_addresses = true;
            //rem -= 2;
        }
        let scrollbar = area.scrollbar(self.scroll, self.line_count());
        let scrollbar_fg = styles.scrollbar_thumb.get_fg()
            .or_else(|| styles.preview.get_fg())
            .unwrap_or(Color::White);
        for y in 0..line_count {
            w.queue(cursor::MoveTo(area.left, y as u16 + area.top))?;
            let mut cw = CropWriter::new(w, area.width as usize - 1); // -1 for scrollbar
            let cw = &mut cw;
            if y < page.len() {
                if addresses {
                    let addr = (self.scroll + y) * 16;
                    cw.queue_g_string(
                        &styles.preview_line_number,
                        match (addresses_len, margin_around_addresses) {
                            (4, false) => format!("{addr:04x}"),
                            (6, false) => format!("{addr:06x}"),
                            (_, false) => format!("{addr:08x}"),
                            (4, true) => format!(" {addr:04x} "),
                            (6, true) => format!(" {addr:06x} "),
                            (_, true) => format!(" {addr:08x} "),
                        },
                    )?;
                }
                if left_margin {
                    cw.queue_char(&styles.default, ' ')?;
                }
                let line = &page[y];
                for x in 0..16 {
                    if x == 8 && hex_middle_space {
                        cw.queue_char(&styles.default, ' ')?;
                    }
                    if let Some(b) = line.bytes.get(x) {
                        let byte = Byte::from(*b);
                        if inter_hex {
                            cw.queue_g_string(byte.style(styles), format!("{b:02x} "))?;
                        } else {
                            cw.queue_g_string(byte.style(styles), format!("{b:02x}"))?;
                        }
                    } else {
                        cw.queue_str(&styles.default, if inter_hex { "   " } else { "  " })?;
                    }
                }
                if chars {
                    cw.queue_char(&styles.default, ' ')?;
                    for x in 0..16 {
                        if x == 8 && chars_middle_space {
                            cw.queue_char(&styles.default, ' ')?;
                        }
                        if let Some(b) = line.bytes.get(x) {
                            let byte = Byte::from(*b);
                            cw.queue_char(byte.style(styles), byte.as_char())?;
                        }
                    }
                }
            }
            cw.fill(&styles.default, &SPACE_FILLING)?;
            if is_thumb(y as u16 + area.top, scrollbar) {
                w.queue(SetForegroundColor(scrollbar_fg))?;
                w.queue(Print('▐'))?;
            } else {
                w.queue(Print(' '))?;
            }
        }
        Ok(())
    }

    pub fn display_info(
        &mut self,
        w: &mut W,
        _screen: Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        let width = area.width as usize;
        let mut s = format!("{}", self.len);
        if s.len() > width {
            return Ok(());
        }
        if s.len() + " bytes".len() < width {
            s = format!("{s} bytes");
        } else if s.len() + 1 < width {
            s = format!("{s}b");
        }
        w.queue(cursor::MoveTo(
            area.left + area.width - s.len() as u16,
            area.top,
        ))?;
        panel_skin.styles.default.queue(w, s)?;
        Ok(())
    }
}

fn is_thumb(y: u16, scrollbar: Option<(u16, u16)>) -> bool {
    if let Some((sctop, scbottom)) = scrollbar {
        if sctop <= y && y <= scbottom {
            return true;
        }
    }
    false
}
