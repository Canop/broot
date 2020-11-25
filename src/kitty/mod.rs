mod kitty_image;

pub use kitty_image::*;

use {
    std::sync::Mutex,
};

lazy_static! {
    static ref RENDERER: Option<Mutex<KittyImageRenderer>> = KittyImageRenderer::new()
        .map(|r| Mutex::new(r));
}

// TODO try to find another way (making app_context mut ?) to pass this
// around without the mutex gymnastic
pub fn image_renderer() -> &'static Option<Mutex<KittyImageRenderer>> {
    &*RENDERER
}

