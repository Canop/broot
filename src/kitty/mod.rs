mod image_renderer;
mod image_set;

pub use {
    image_renderer::*,
    image_set::*,
};

use {
    crate::{
        display::W,
        errors::ProgramError,
    },
    image::DynamicImage,
    once_cell::sync::Lazy,
    std::sync::Mutex,
    termimad::Area,
};

/// Give the current images, so that they can be removed
/// (which should be done only after a new content has been
/// displayed)
pub fn take_current_images() -> Option<KittyImageSet> {
    manager().lock().unwrap().take_current_images()
}
/// Try print the image in the specified area.
///
/// Return Ok(true) if it went well, Ok(false) if
/// the terminal doesn't appear compatible with the Kitty
/// graphics protocol, or an error if somebody went wrong.
pub fn try_print_image(
    w: &mut W,
    src: &DynamicImage,
    area: &Area,
) -> Result<bool, ProgramError> {
    let mut manager = manager().lock().unwrap();
    manager.try_print_image(w, src, area)
}

static MANAGER: Lazy<Mutex<KittyManager>> = Lazy::new(|| {
    let manager = KittyManager {
        current_images: None,
        renderer: MaybeRenderer::Untested,
    };
    Mutex::new(manager)
});

fn manager() -> &'static Mutex<KittyManager> {
    &*MANAGER
}

#[derive(Debug)]
struct KittyManager {
    current_images: Option<KittyImageSet>,
    renderer: MaybeRenderer,
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
    pub fn take_current_images(&mut self) -> Option<KittyImageSet> {
        self.current_images.take()
    }
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
    pub fn try_print_image(
        &mut self,
        w: &mut W,
        src: &DynamicImage,
        area: &Area,
    ) -> Result<bool, ProgramError> {
        if let Some(renderer) = self.renderer() {
            let new_id = renderer.print(w, src, area)?;
            self.current_images
                .get_or_insert_with(KittyImageSet::default)
                .push(new_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
