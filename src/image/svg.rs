use {
    crate::{
        errors::SvgError,
    },
    image::{
        DynamicImage,
        RgbaImage,
    },
    std::path::PathBuf,
    usvg::ScreenSize,
    usvg_text_layout::{fontdb, TreeTextToPath},
};

fn compute_zoom(width:u32, height:u32, max_width:u32, max_height:u32) -> Result<f32, SvgError> {
    let w: f32 = width as f32;
    let h: f32 = height as f32;
    let mw: f32 = max_width.max(2) as f32;
    let mh: f32 = max_height.max(2) as f32;
    let zoom = 1.0f32
        .min(mw / w)
        .min(mh / h);
    if zoom > 0.0f32 {
        Ok(zoom)
    } else {
        Err(SvgError::Internal { message: "invalid SVG dimensions" })
    }
}

/// Generate a bitmap at the natural dimensions of the SVG unless it's too big
///  in which case a smaller one is generated to fit into (max_width x max_height).
pub fn render<P: Into<PathBuf>>(
    path: P,
    max_width: u32,
    max_height: u32,
) -> Result<DynamicImage, SvgError> {
    let path: PathBuf = path.into();
    let mut opt = usvg::Options::default();
    opt.resources_dir = Some(path.clone());
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();
    let svg_data = std::fs::read(path)?;
    let mut tree = usvg::Tree::from_data(&svg_data, &opt)?;
    debug!("SVG natural size: {} x {}", tree.size.width(), tree.size.height());
    let px_size = tree.size.to_screen_size();
    let zoom = compute_zoom(px_size.width(), px_size.height(), max_width, max_height)?;
    debug!("svg rendering zoom: {zoom}");
    let Some(px_size) = ScreenSize::new(
        (px_size.width() as f32 * zoom) as u32,
        (px_size.height() as f32 * zoom) as u32,
    ) else {
        return Err(SvgError::Internal { message: "invalid SVG dimensions" });
    };
    debug!("px_size: {px_size:?}");
    tree.convert_text(&fontdb, opt.keep_named_groups);
    let mut pixmap = tiny_skia::Pixmap::new(
        px_size.width(),
        px_size.height(),
    ).ok_or(SvgError::Internal { message: "unable to create pixmap buffer" })?;
    resvg::render(
        &tree,
        usvg::FitTo::Zoom(zoom),
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    ).ok_or(SvgError::Internal { message: "resvg doesn't look happy (not sure)" })?;
    let image_buffer = RgbaImage::from_vec(
        pixmap.width(),
        pixmap.height(),
        pixmap.take(),
    ).ok_or(SvgError::Internal { message: "wrong image buffer size" })?;
    Ok(DynamicImage::ImageRgba8(image_buffer))
}
