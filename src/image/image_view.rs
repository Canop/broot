use {
    crate::{
        display::{fill_bg, Screen, W},
        errors::ProgramError,
        skin::PanelSkin,
    },
    crossterm::{
        cursor,
        style::{
            Attributes,
            Color,
            ContentStyle,
            PrintStyledContent,
            SetBackgroundColor,
            StyledContent,
        },
        QueueableCommand,
    },
    image::{
        io::Reader,
        DynamicImage,
        GenericImageView,
        imageops::FilterType,
        Rgba,
    },
    std::{
        path::Path,
    },
    termimad::{Area},
};

pub struct ImageView {
    img: DynamicImage,
}

const UPPER_HALF_BLOCK: char = '▀';

// Each char row covers two pixel lines.
struct DoubleLine {
    img_width: usize,
    pixels: Vec<Color>, // size twice img_width
}
impl DoubleLine {
    fn new(img_width: usize) -> Self {
        Self {
            img_width,
            pixels: Vec::with_capacity(2 * img_width),
        }
    }
    fn push(&mut self, rgba: Rgba<u8>) {
        self.pixels.push(Color::Rgb{
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
        });
    }
    fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }
    fn is_full(&self) -> bool {
        self.pixels.len() == 2 * self.img_width
    }
    fn write(
        &mut self,
        w: &mut W,
        left_margin: usize,
        right_margin: usize,
        bg: Color,
    ) -> Result<(), ProgramError> {
        debug_assert!(self.pixels.len()==self.img_width || self.pixels.len() == 2*self.img_width);
        // we may have either one or two lines
        let simple = self.pixels.len() < 2 * self.img_width;
        fill_bg(w, left_margin, bg)?;
        for i in 0..self.img_width {
            let foreground_color = Some(self.pixels[i]);
            let background_color = if simple {
                None
            } else {
                Some(self.pixels[i + self.img_width])
            };
            w.queue(PrintStyledContent(StyledContent::new(
                ContentStyle {
                    foreground_color,
                    background_color,
                    attributes: Attributes::default(),
                },
                UPPER_HALF_BLOCK,
            )))?;
        }
        fill_bg(w, right_margin, bg)?;
        self.pixels.clear();
        Ok(())
    }
}

// TODO disable if not true color ?
impl ImageView {
    pub fn new(path: &Path) -> Result<Self, ProgramError> {
        let img = Reader::open(&path)?
            .decode()?;
        let (width, height) = img.dimensions();
        debug!("image dimensions: {},{}", width, height);
        Ok(Self {
            img,
        })
    }
    pub fn display(
        &mut self,
        w: &mut W,
        _screen: &Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        debug!("img view area: {:?}", area);
        let img = self.img.resize(
            area.width as u32,
            (area.height*2) as u32,
            FilterType::Triangle,
            //FilterType::Nearest,
        );
        let (width, height) = img.dimensions();
        debug!("resized image dimensions: {},{}", width, height);
        debug_assert!(width <= area.width as u32);
        let styles = &panel_skin.styles;
        let bg = styles.preview.get_bg()
            .or_else(|| styles.default.get_bg())
            .unwrap_or(Color::AnsiValue(238));
        //let mut (x, y) = (0, 0);
        let mut double_line = DoubleLine::new(width as usize);
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
}


