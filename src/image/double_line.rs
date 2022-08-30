use {
    crate::{
        display::W,
        errors::ProgramError,
    },
    ansi_colours,
    crokey::crossterm::{
        style::{
            Color,
            Colors,
            Print,
            SetColors,
        },
        QueueableCommand,
    },
    image::Rgba,
    termimad::fill_bg,
};

const UPPER_HALF_BLOCK: char = '▀';

/// A "double line" normally contains two lines of pixels
/// which are displayed as one line of characters, the
/// UPPER_HALF_BLOCK foreground for the upper pixel and
/// the background of the char for the lower pixel.
/// It acts as a buffer which is dumped to screen when
/// full or when the image is totally read.
pub struct DoubleLine {
    img_width: usize,
    pixels: Vec<Color>, // size twice img_width
    true_colors: bool,
}

impl DoubleLine {
    pub fn new(img_width: usize, true_colors: bool) -> Self {
        Self {
            img_width,
            pixels: Vec::with_capacity(2 * img_width),
            true_colors,
        }
    }
    pub fn push(&mut self, rgba: Rgba<u8>) {
        self.pixels.push(
            if self.true_colors {
                Color::Rgb {
                    r: rgba[0],
                    g: rgba[1],
                    b: rgba[2],
                }
            } else {
                Color::AnsiValue(ansi_colours::ansi256_from_rgb((
                    rgba[0],
                    rgba[1],
                    rgba[2],
                )))
            }
        );
    }
    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }
    pub fn is_full(&self) -> bool {
        self.pixels.len() == 2 * self.img_width
    }
    pub fn write(
        &mut self,
        w: &mut W,
        left_margin: usize,
        right_margin: usize,
        bg: Color,
    ) -> Result<(), ProgramError> {
        debug_assert!(
            self.pixels.len() == self.img_width || self.pixels.len() == 2 * self.img_width
        );
        // we may have either one or two lines
        let simple = self.pixels.len() < 2 * self.img_width;
        fill_bg(w, left_margin, bg)?;
        for i in 0..self.img_width {
            let foreground_color = self.pixels[i];
            let background_color = if simple {
                bg
            } else {
                self.pixels[i + self.img_width]
            };
            w.queue(SetColors(Colors::new(
                foreground_color,
                background_color,
            )))?;
            w.queue(Print(UPPER_HALF_BLOCK))?;
        }
        fill_bg(w, right_margin, bg)?;
        self.pixels.clear();
        Ok(())
    }
}

