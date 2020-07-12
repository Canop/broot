
use {
    super::{
        byte::Byte,
    },
    crate::{
        display::{CropWriter, LONG_SPACE, Screen, W},
        errors::ProgramError,
        skin::PanelSkin,
    },
    crossterm::{
        cursor,
        QueueableCommand,
    },
    memmap::Mmap,
    std::{
        fs::File,
        io,
        path::PathBuf,
    },
    termimad::{Area},
};


pub struct HexLine {
    pub bytes: Vec<u8>, // from 1 to 16 bytes
}

pub struct HexView {
    pub path: PathBuf,
    pub scroll: usize,
    pub len: usize,
}

impl HexView {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let len = path.metadata()?.len() as usize;
        Ok(Self {
            path,
            scroll: 0,
            len,
        })
    }
    pub fn line_count(&self) -> usize {
        self.len / 16
    }
    pub fn get_page(&mut self, start_line_idx: usize, line_count: usize) -> io::Result<Vec<HexLine>> {
        // I'm not sure a memmap is the best solution here but at least it's easy
        let file = File::open(&self.path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let new_len = mmap.len();
        if new_len != self.len {
            warn!("previewed file len changed from {} to {}", self.len, new_len);
            self.len = new_len;
        }
        let mut lines = Vec::new();
        let mut start_idx = 16 * start_line_idx;
        while start_idx < self.len {
            let line_len = 16.min(self.len - start_idx);
            let mut bytes: Vec<u8> = vec![0; line_len];
            bytes[0..line_len].copy_from_slice(&mmap[start_idx..start_idx+line_len]);
            lines.push(HexLine{bytes});
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
        _screen: &Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        let line_count = area.height as usize;
        let page = self.get_page(self.scroll, line_count)?;
        let styles = &panel_skin.styles;
        let mut show_middle_space = false;
        if area.width > 50 {
            show_middle_space = true;
        }
        for y in 0..line_count {
            w.queue(cursor::MoveTo(area.left, y as u16 + area.top))?;
            let mut cw = CropWriter::new(w, area.width as usize);
            let cw = &mut cw;
            if y < page.len() {
                let line = &page[y];
                for x in 0..16 {
                    if x==8 && show_middle_space {
                        cw.queue_char(&styles.default, ' ')?;
                    }
                    if let Some(b) = line.bytes.get(x) {
                        let byte = Byte::from(*b);
                        cw.queue_string(byte.style(styles), format!(" {:02x}", b))?;
                    } else {
                        cw.queue_str(&styles.default, "   ")?;
                    }
                }
                cw.fill(&styles.default, LONG_SPACE)?;
            }
        }
        Ok(())
    }
}
