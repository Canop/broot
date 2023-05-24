mod detect_support;
mod image_renderer;

pub use {
    image_renderer::*,
};

use {
    crate::{
        display::W,
        errors::ProgramError,
        image::SourceImage,
    },
    crokey::crossterm::style::Color,
    once_cell::sync::Lazy,
    std::{
        io::Write,
        sync::Mutex,
    },
    termimad::Area,
};

pub type KittyImageId = usize;

static MANAGER: Lazy<Mutex<KittyManager>> = Lazy::new(|| {
    let manager = KittyManager {
        rendered_images: Vec::new(),
        renderer: MaybeRenderer::Untested,
    };
    Mutex::new(manager)
});

pub fn manager() -> &'static Mutex<KittyManager> {
    &MANAGER
}

#[derive(Debug)]
pub struct KittyManager {
    rendered_images: Vec<RenderedImage>,
    renderer: MaybeRenderer,
}

#[derive(Debug)]
struct RenderedImage {
    image_id: KittyImageId,
    drawing_count: usize,
}

#[derive(Debug)]
enum MaybeRenderer {
    Untested,
    Disabled,
    Enabled {
        renderer: KittyImageRenderer,
    },
}

impl KittyManager {
    /// return the renderer if it's already checked and enabled, none if
    /// it's disabled or if it hasn't been tested yet
    pub fn renderer_if_tested(&mut self) -> Option<&mut KittyImageRenderer> {
        match &mut self.renderer {
            MaybeRenderer::Enabled { renderer } => Some(renderer),
            _ => None,
        }
    }
    pub fn renderer(&mut self) -> Option<&mut KittyImageRenderer> {
        if matches!(self.renderer, MaybeRenderer::Disabled) {
            return None;
        }
        if matches!(self.renderer, MaybeRenderer::Enabled { .. }) {
            return self.renderer_if_tested();
        }
        // we're in the Untested branch
        match KittyImageRenderer::new() {
            Some(renderer) => {
                self.renderer = MaybeRenderer::Enabled { renderer };
                self.renderer_if_tested()
            }
            None => {
                self.renderer = MaybeRenderer::Disabled;
                None
            }
        }
    }
    pub fn keep(
        &mut self,
        kept_id: KittyImageId,
        drawing_count: usize,
    ) {
        for image in self.rendered_images.iter_mut() {
            if image.image_id == kept_id {
                image.drawing_count = drawing_count;
            }
        }
    }
    pub fn try_print_image(
        &mut self,
        w: &mut W,
        src: &SourceImage,
        area: &Area,
        bg: Color,
        drawing_count: usize,
    ) -> Result<Option<KittyImageId>, ProgramError> {
        if let Some(renderer) = self.renderer() {
            let img = src.optimal()?;
            let new_id = renderer.print(w, &img, area, bg)?;
            self.rendered_images.push(RenderedImage {
                image_id: new_id,
                drawing_count,
            });
            Ok(Some(new_id))
        } else {
            Ok(None)
        }
    }
    pub fn erase_images_before(
        &mut self,
        w: &mut W,
        drawing_count: usize,
    ) -> Result<(), ProgramError> {
        let mut kept_images = Vec::new();
        for image in self.rendered_images.drain(..) {
            if image.drawing_count >= drawing_count {
                kept_images.push(image);
            } else {
                let id = image.image_id;
                debug!("erase kitty image {}", id);
                write!(w, "\u{1b}_Ga=d,d=I,i={id}\u{1b}\\")?;
            }
        }
        self.rendered_images = kept_images;
        Ok(())
    }
}
