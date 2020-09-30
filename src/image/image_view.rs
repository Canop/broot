use {
    super::double_line::DoubleLine,
    crate::{
        app::AppContext,
        display::{fill_bg, Screen, W},
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
    std::path::Path,
    termimad::{Area},
};

/// an imageview can display an image in the terminal with
/// a ration of one pixel per char in width.
pub struct ImageView {
    img: DynamicImage,
}

impl ImageView {
    pub fn new(path: &Path) -> Result<Self, ProgramError> {
        let img = time!(
            Debug,
            "decode image",
            path,
            Reader::open(&path)?.decode()?
        );
        let (width, height) = img.dimensions();
        debug!("image dimensions: {},{}", width, height);
        Ok(Self {
            img,
        })
    }
    pub fn display(
        &mut self,
        w: &mut W,
        _screen: Screen,
        panel_skin: &PanelSkin,
        area: &Area,
        con: &AppContext,
    ) -> Result<(), ProgramError> {
        let img = time!(
            Debug,
            "resize image",
            self.img.resize(
                area.width as u32,
                (area.height*2) as u32,
                FilterType::Triangle,
            ),
        );
        let (width, height) = img.dimensions();
        debug!("resized image dimensions: {},{}", width, height);
        debug_assert!(width <= area.width as u32);
        let styles = &panel_skin.styles;
        let bg = styles.preview.get_bg()
            .or_else(|| styles.default.get_bg())
            .unwrap_or(Color::AnsiValue(238));
        let mut double_line = DoubleLine::new(width as usize, con.true_colors);
        let mut y = area.top;
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
        for y in y..area.top+area.height {
            w.queue(cursor::MoveTo(area.left, y))?;
            fill_bg(w,area.width as usize, bg)?;
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
        let dim = self.img.dimensions();
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


