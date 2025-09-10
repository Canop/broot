use {
    crate::errors::SvgError,
    image::{
        DynamicImage,
        RgbaImage,
    },
    resvg::{
        tiny_skia,
        usvg,
    },
    std::path::PathBuf,
    termimad::coolor,
};

fn compute_zoom(
    w: f32,
    h: f32,
    max_width: u32,
    max_height: u32,
) -> Result<f32, SvgError> {
    let mw: f32 = max_width.max(2) as f32;
    let mh: f32 = max_height.max(2) as f32;
    let zoom = 1.0f32.min(mw / w).min(mh / h);
    if zoom > 0.0f32 {
        Ok(zoom)
    } else {
        Err(SvgError::Internal {
            message: "invalid SVG dimensions",
        })
    }
}

pub fn load<P: Into<PathBuf>>(path: P) -> Result<usvg::Tree, SvgError> {
    let path: PathBuf = path.into();
    let mut opt = usvg::Options {
        resources_dir: Some(path.clone()),
        ..Default::default()
    };
    opt.fontdb_mut().load_system_fonts();
    let svg_data = std::fs::read(path)?;
    let tree = usvg::Tree::from_data(&svg_data, &opt)?;
    Ok(tree)
}
pub fn render_tree(
    tree: &usvg::Tree,
    max_width: u32,
    max_height: u32,
    bg_color: Option<coolor::Color>,
) -> Result<DynamicImage, SvgError> {
    let t_width = tree.size().width();
    let t_height = tree.size().height();
    debug!("SVG natural size: {t_width} x {t_height}");
    let zoom = compute_zoom(t_width, t_height, max_width, max_height)?;
    debug!("svg rendering zoom: {zoom}");
    let px_width = (t_width * zoom) as u32;
    let px_height = (t_height * zoom) as u32;
    if px_width == 0 || px_height == 0 {
        return Err(SvgError::Internal {
            message: "invalid SVG dimensions",
        });
    };
    debug!("px_size: ({px_width}, {px_height})");
    let mut pixmap =
        tiny_skia::Pixmap::new(px_width, px_height).ok_or(SvgError::Internal {
            message: "unable to create pixmap buffer",
        })?;
    if let Some(bg_color) = bg_color {
        let rgb = bg_color.rgb();
        let bg_color = tiny_skia::Color::from_rgba8(rgb.r, rgb.g, rgb.b, 255);
        pixmap.fill(bg_color);
    }
    resvg::render(
        tree,
        tiny_skia::Transform::from_scale(zoom, zoom),
        &mut pixmap.as_mut(),
    );
    let image_buffer =
        RgbaImage::from_vec(pixmap.width(), pixmap.height(), pixmap.take()).ok_or(
            SvgError::Internal {
                message: "wrong image buffer size",
            },
        )?;
    Ok(DynamicImage::ImageRgba8(image_buffer))
}

/// Generate a bitmap at the natural dimensions of the SVG unless it's too big
///  in which case a smaller one is generated to fit into (max_width x max_height).
///
/// Background will be black.
#[allow(dead_code)]
pub fn render<P: Into<PathBuf>>(
    path: P,
    max_width: u32,
    max_height: u32,
) -> Result<DynamicImage, SvgError> {
    let tree = load(path)?;
    let image = render_tree(&tree, max_width, max_height, None)?;
    Ok(image)
}
