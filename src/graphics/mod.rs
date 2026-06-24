mod fit;
pub mod image_data;
pub mod terminal;

pub use fit::{rendering_area, rendering_dim};

use {
    crate::{display::W, errors::ProgramError, image::zune_compat::DynamicImage},
    crokey::crossterm::style::Color,
    std::path::Path,
    termimad::Area,
};

pub type ImageId = usize;

/// A terminal-graphics protocol able to draw an image into a screen area.
pub trait GraphicsRenderer: Send {
    /// Clear `area` and draw `src` (already fitted to the area's pixels) into it.
    /// Returns `Some(id)` for protocols whose images persist above the cells and
    /// must be erased later (Kitty); `None` for protocols drawn into the cells and
    /// cleared by normal repaint (Sixel).
    fn print(
        &mut self,
        w: &mut W,
        src: &DynamicImage,
        src_path: &Path,
        area: &Area,
        bg: Color,
    ) -> Result<Option<ImageId>, ProgramError>;

    /// Erase a previously drawn image by id. No-op for inline protocols.
    fn erase_image(&self, w: &mut W, id: ImageId) -> Result<(), ProgramError>;

    fn delete_temp_files(&mut self) {}
}

use {
    crate::app::AppContext,
    crate::display::cell_size_in_pixels,
    once_cell::sync::Lazy,
    std::sync::Mutex,
};

static MANAGER: Lazy<Mutex<GraphicsManager>> = Lazy::new(|| {
    Mutex::new(GraphicsManager {
        rendered_images: Vec::new(),
        renderer: MaybeRenderer::Untested,
    })
});

pub fn manager() -> &'static Mutex<GraphicsManager> {
    &MANAGER
}

#[derive(Debug)]
struct RenderedImage {
    image_id: ImageId,
    drawing_count: usize,
}

enum MaybeRenderer {
    Untested,
    Disabled,
    Enabled { renderer: Box<dyn GraphicsRenderer> },
}

pub struct GraphicsManager {
    rendered_images: Vec<RenderedImage>,
    renderer: MaybeRenderer,
}

impl GraphicsManager {
    fn renderer_if_tested(&mut self) -> Option<&mut dyn GraphicsRenderer> {
        match &mut self.renderer {
            MaybeRenderer::Enabled { renderer } => Some(renderer.as_mut()),
            _ => None,
        }
    }
    pub fn delete_temp_files(&mut self) {
        if let MaybeRenderer::Enabled { renderer } = &mut self.renderer {
            renderer.delete_temp_files();
        }
    }
    pub fn renderer(&mut self, con: &AppContext) -> Option<&mut dyn GraphicsRenderer> {
        if matches!(self.renderer, MaybeRenderer::Disabled) {
            return None;
        }
        if matches!(self.renderer, MaybeRenderer::Enabled { .. }) {
            return self.renderer_if_tested();
        }
        // protocol selection added in Task 6; for now: Kitty only
        match crate::kitty::build_kitty_renderer(con) {
            Some(renderer) => {
                self.renderer = MaybeRenderer::Enabled {
                    renderer: Box::new(renderer),
                };
                self.renderer_if_tested()
            }
            None => {
                self.renderer = MaybeRenderer::Disabled;
                None
            }
        }
    }
    pub fn keep(&mut self, kept_id: ImageId, drawing_count: usize) {
        for image in &mut self.rendered_images {
            if image.image_id == kept_id {
                image.drawing_count = drawing_count;
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn try_print_image(
        &mut self,
        w: &mut W,
        src: &crate::image::SourceImage,
        src_path: &Path,
        area: &Area,
        bg: Color,
        drawing_count: usize,
        con: &AppContext,
    ) -> Result<Option<ImageId>, ProgramError> {
        if let Ok((cell_width, cell_height)) = cell_size_in_pixels() {
            if let Some(renderer) = self.renderer(con) {
                let area_width = area.width as u32 * cell_width;
                let area_height = area.height as u32 * cell_height;
                let img = src.fitting(area_width, area_height, None)?;
                if let Some(new_id) = renderer.print(w, &img, src_path, area, bg)? {
                    self.rendered_images.push(RenderedImage {
                        image_id: new_id,
                        drawing_count,
                    });
                    return Ok(Some(new_id));
                }
                return Ok(None);
            }
        }
        Ok(None)
    }
    pub fn erase_images_before(
        &mut self,
        w: &mut W,
        drawing_count: usize,
    ) -> Result<(), ProgramError> {
        let mut kept_images = Vec::new();
        let stale: Vec<RenderedImage> = self.rendered_images.drain(..).collect();
        for image in stale {
            if image.drawing_count >= drawing_count {
                kept_images.push(image);
            } else if let MaybeRenderer::Enabled { renderer } = &self.renderer {
                renderer.erase_image(w, image.image_id)?;
            }
        }
        self.rendered_images = kept_images;
        Ok(())
    }
}
