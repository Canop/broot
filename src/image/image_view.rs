use {
    super::double_line::DoubleLine,
    crate::{
        app::AppContext,
        display::{Screen, W},
        errors::ProgramError,
        skin::PanelSkin,
    },
    crossterm::{
        cursor,
        style::{
            Color,
            SetBackgroundColor,
        },
        QueueableCommand,
    },
    image::{
        io::Reader,
        DynamicImage,
        GenericImageView,
        imageops::FilterType,
    },
    std::path::{Path, PathBuf},
    termimad::{fill_bg, Area},
};

/// an already resized image, with the dimensions it
/// was computed for (which may be different from the
/// dimensions we got)
struct CachedImage {
    img: DynamicImage,
    target_width: u32,
    target_height: u32,
}

/// an imageview can display an image in the terminal with
/// a ratio of one pixel per char in width.
pub struct ImageView {
    path: PathBuf,
    source_img: DynamicImage,
    display_img: Option<CachedImage>,
}

impl ImageView {
    pub fn new(path: &Path) -> Result<Self, ProgramError> {
        let source_img = time!(
            "decode image",
            path,
            Reader::open(&path)?.decode()?
        );
        Ok(Self {
            path: path.to_path_buf(),
            source_img,
            display_img: None,
        })
    }
    pub fn is_png(&self) -> bool {
        match self.path.extension() {
            Some(ext) => ext == "png" || ext == "PNG",
            None => false,
        }
    }
    pub fn display(
        &mut self,
        w: &mut W,
        _screen: Screen,
        panel_skin: &PanelSkin,
        area: &Area,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        let styles = &panel_skin.styles;
        let bg = styles.preview.get_bg()
            .or_else(|| styles.default.get_bg())
            .unwrap_or(Color::AnsiValue(238));

        if crate::kitty::try_print_image(w, &self.source_img, area)? {
            return Ok(());
        }

        let target_width = area.width as u32;
        let target_height = (area.height * 2) as u32;
        let cached = self
            .display_img
            .as_ref()
            .filter(|ci| ci.target_width == target_width && ci.target_height == target_height);
        let img = match cached {
            Some(ci) => &ci.img,
            None => {
                let img = time!(
                    "resize image",
                    self.source_img.resize(target_width, target_height, FilterType::Triangle),
                );
                self.display_img = Some(CachedImage {
                    img,
                    target_width,
                    target_height,
                });
                &self.display_img.as_ref().unwrap().img
            }
        };
        let (width, height) = img.dimensions();
        debug!("resized image dimensions: {},{}", width, height);
        debug_assert!(width <= area.width as u32);
        let mut double_line = DoubleLine::new(width as usize, con.true_colors);
        let mut y = area.top;
        let img_top_offset = (area.height - (height / 2) as u16) / 2;
        for _ in 0..img_top_offset {
            w.queue(cursor::MoveTo(area.left, y))?;
            fill_bg(w, area.width as usize, bg)?;
            y += 1;
        }
        let margin = area.width as usize - width as usize;
        let left_margin = margin / 2;
        let right_margin = margin - left_margin;
        w.queue(cursor::MoveTo(area.left, y))?;
        for pixel in img.pixels() {
            double_line.push(pixel.2);
            if double_line.is_full() {
                double_line.write(w, left_margin, right_margin, bg)?;
                y += 1;
                w.queue(cursor::MoveTo(area.left, y))?;
            }
        }
        if !double_line.is_empty() {
            double_line.write(w, left_margin, right_margin, bg)?;
            y += 1;
        }
        w.queue(SetBackgroundColor(bg))?;
        for y in y..area.top + area.height {
            w.queue(cursor::MoveTo(area.left, y))?;
            fill_bg(w, area.width as usize, bg)?;
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
        let dim = self.source_img.dimensions();
        let s = format!("{} x {}", dim.0, dim.1);
        if s.len() > area.width as usize {
            return Ok(());
        }
        w.queue(cursor::MoveTo(
            area.left + area.width - s.len() as u16,
            area.top,
        ))?;
        panel_skin.styles.default.queue(w, s)?;
        Ok(())
    }
}

