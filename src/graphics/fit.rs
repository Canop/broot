use termimad::Area;

fn div_ceil(a: u32, b: u32) -> u32 {
    a / b + u32::from(0 != a % b)
}

/// Compute the on-screen size in cells for an image, fitting it into the area.
pub fn rendering_dim(
    cell_width: u32,
    cell_height: u32,
    img_width: u32,
    img_height: u32,
    area_cols: u32,
    area_rows: u32,
) -> (u32, u32) {
    // Invariants that keep the divisions below safe: cell size is nonzero by
    // construction (terminal detection rejects zero) and decoded images have
    // nonzero dimensions, so optimal_cols/optimal_rows are >= 1.
    debug_assert!(cell_width != 0 && cell_height != 0, "cell size must be nonzero");
    debug_assert!(img_width != 0 && img_height != 0, "image dimensions must be nonzero");
    let optimal_cols = div_ceil(img_width, cell_width);
    let optimal_rows = div_ceil(img_height, cell_height);
    if optimal_cols <= area_cols && optimal_rows <= area_rows {
        (optimal_cols, optimal_rows)
    } else if optimal_cols * area_rows > optimal_rows * area_cols {
        (area_cols, optimal_rows * area_cols / optimal_cols)
    } else {
        (optimal_cols * area_rows / optimal_rows, area_rows)
    }
}

/// Centered sub-area (in cells) for the image inside `area`.
pub fn rendering_area(
    cell_width: u32,
    cell_height: u32,
    img_width: u32,
    img_height: u32,
    area: &Area,
) -> Area {
    let area_cols: u32 = area.width.into();
    let area_rows: u32 = area.height.into();
    let rdim = rendering_dim(cell_width, cell_height, img_width, img_height, area_cols, area_rows);
    Area::new(
        area.left + ((area_cols - rdim.0) / 2) as u16,
        area.top + ((area_rows - rdim.1) / 2) as u16,
        rdim.0 as u16,
        rdim.1 as u16,
    )
}

#[cfg(test)]
mod tests {
    use super::rendering_dim;

    #[test]
    fn unconstrained_uses_optimal_cells() {
        // 20x20 image, 10x10 cells, big area -> 2x2 cells
        assert_eq!(rendering_dim(10, 10, 20, 20, 100, 100), (2, 2));
    }

    #[test]
    fn width_constrained_scales_down() {
        // wide image, narrow area -> clamps to area_cols
        let (cols, rows) = rendering_dim(10, 10, 1000, 100, 5, 100);
        assert_eq!(cols, 5);
        assert!(rows <= 100);
    }

    #[test]
    fn height_constrained_scales_down() {
        let (cols, rows) = rendering_dim(10, 10, 100, 1000, 100, 5);
        assert_eq!(rows, 5);
        assert!(cols <= 100);
    }
}
