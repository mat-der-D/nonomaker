use image::{DynamicImage, GenericImageView, GrayImage, Luma, imageops::FilterType};

use crate::types::{Cell, Grid};

#[derive(Debug, Clone)]
pub struct ImageConvertParams {
    pub grid_width: u32,
    pub grid_height: u32,
    pub smooth_strength: f32,
    pub threshold: u8,
    pub edge_strength: f32,
    pub noise_removal: u32,
}

impl Default for ImageConvertParams {
    fn default() -> Self {
        Self {
            grid_width: 20,
            grid_height: 20,
            smooth_strength: 1.0,
            threshold: 128,
            edge_strength: 0.3,
            noise_removal: 0,
        }
    }
}

#[derive(Debug)]
pub enum ImageError {
    Decode(image::ImageError),
    InvalidParams(String),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decode(e) => write!(f, "image decode failed: {e}"),
            Self::InvalidParams(msg) => write!(f, "invalid image convert params: {msg}"),
        }
    }
}

impl std::error::Error for ImageError {}

impl From<image::ImageError> for ImageError {
    fn from(value: image::ImageError) -> Self {
        Self::Decode(value)
    }
}

pub fn image_to_grid(image_bytes: &[u8], params: &ImageConvertParams) -> Result<Grid, ImageError> {
    validate_params(params)?;

    let decoded = image::load_from_memory(image_bytes)?;
    let resized = resize_to_fit(decoded, 384, 384);
    let gray = alpha_composite_to_gray(&resized);

    let blur_buf = apply_blur(&gray, params.smooth_strength);
    let blurred = blur_buf.as_ref().unwrap_or(&gray);

    let edges = if params.edge_strength > 0.0 {
        imageproc::edges::canny(blurred, 50.0, 150.0)
    } else {
        GrayImage::new(blurred.width(), blurred.height())
    };

    let merged = merge_edge(blurred, &edges, params.edge_strength.clamp(0.0, 1.0));

    let grid_w = params.grid_width as usize;
    let grid_h = params.grid_height as usize;
    let cell_averages = downsample(&merged, grid_w, grid_h);

    let threshold = f32::from(params.threshold);
    let mut cells: Vec<Vec<bool>> = cell_averages
        .iter()
        .map(|row| row.iter().map(|&v| v < threshold).collect())
        .collect();

    if params.noise_removal > 0 {
        cells = remove_noise(cells, grid_w, grid_h, params.noise_removal as usize);
    }

    Ok(cells_to_grid(&cells, grid_w, grid_h))
}

fn cells_to_grid(cells: &[Vec<bool>], width: usize, height: usize) -> Grid {
    let mut grid = Grid::new(width, height);
    for (r, row) in cells.iter().enumerate() {
        for (c, &filled) in row.iter().enumerate() {
            *grid.cell_mut(r, c) = if filled { Cell::Filled } else { Cell::Blank };
        }
    }
    grid
}

fn validate_params(params: &ImageConvertParams) -> Result<(), ImageError> {
    if params.grid_width == 0 || params.grid_height == 0 {
        return Err(ImageError::InvalidParams(
            "grid_width and grid_height must be positive".to_string(),
        ));
    }
    if !(0.0..=5.0).contains(&params.smooth_strength) {
        return Err(ImageError::InvalidParams(
            "smooth_strength must be in [0, 5]".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&params.edge_strength) {
        return Err(ImageError::InvalidParams(
            "edge_strength must be in [0, 1]".to_string(),
        ));
    }
    Ok(())
}

fn resize_to_fit(img: DynamicImage, max_w: u32, max_h: u32) -> DynamicImage {
    let (w, h) = img.dimensions();
    if w <= max_w && h <= max_h {
        return img;
    }

    let scale = (max_w as f32 / w as f32).min(max_h as f32 / h as f32);
    let target_w = ((w as f32 * scale).floor() as u32).max(1);
    let target_h = ((h as f32 * scale).floor() as u32).max(1);

    let rgba = img.to_rgba8();
    let resized = image::imageops::resize(&rgba, target_w, target_h, FilterType::Triangle);
    DynamicImage::ImageRgba8(resized)
}

fn apply_blur(img: &GrayImage, strength: f32) -> Option<GrayImage> {
    if strength > 0.0 {
        Some(imageproc::filter::gaussian_blur_f32(img, strength))
    } else {
        None
    }
}

fn alpha_composite_to_gray(img: &DynamicImage) -> GrayImage {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let mut gray = GrayImage::new(w, h);

    for (x, y, pixel) in rgba.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;
        let alpha = a as f32 / 255.0;
        let composite = |ch: u8| ch as f32 * alpha + 255.0 * (1.0 - alpha);
        let luma = (0.299 * composite(r) + 0.587 * composite(g) + 0.114 * composite(b)) as u8;
        gray.put_pixel(x, y, Luma([luma]));
    }

    gray
}

fn merge_edge(gray: &GrayImage, edges: &GrayImage, edge_strength: f32) -> GrayImage {
    let (w, h) = gray.dimensions();
    let mut merged = GrayImage::new(w, h);

    for y in 0..h {
        for x in 0..w {
            let gv = f32::from(gray.get_pixel(x, y).0[0]);
            let ev = f32::from(edges.get_pixel(x, y).0[0]);
            let mv = (gv - ev * edge_strength).clamp(0.0, 255.0) as u8;
            merged.put_pixel(x, y, Luma([mv]));
        }
    }

    merged
}

fn downsample(img: &GrayImage, grid_w: usize, grid_h: usize) -> Vec<Vec<f32>> {
    let iw = img.width() as usize;
    let ih = img.height() as usize;
    let cell_w = iw / grid_w;
    let cell_h = ih / grid_h;

    (0..grid_h)
        .map(|row| {
            (0..grid_w)
                .map(|col| cell_average(img, row, col, cell_w, cell_h))
                .collect()
        })
        .collect()
}

fn cell_average(img: &GrayImage, row: usize, col: usize, cell_w: usize, cell_h: usize) -> f32 {
    let x0 = col * cell_w;
    let x1 = x0 + cell_w;
    let y0 = row * cell_h;
    let y1 = y0 + cell_h;

    let mut sum = 0u64;
    let mut count = 0u64;
    for y in y0..y1 {
        for x in x0..x1 {
            sum += u64::from(img.get_pixel(x as u32, y as u32).0[0]);
            count += 1;
        }
    }

    if count > 0 {
        sum as f32 / count as f32
    } else {
        255.0
    }
}

fn neighbors_4(
    r: usize,
    c: usize,
    height: usize,
    width: usize,
) -> impl Iterator<Item = (usize, usize)> {
    let mut arr = [(0usize, 0usize); 4];
    let mut n = 0;

    if r > 0 {
        arr[n] = (r - 1, c);
        n += 1;
    }
    if r + 1 < height {
        arr[n] = (r + 1, c);
        n += 1;
    }
    if c > 0 {
        arr[n] = (r, c - 1);
        n += 1;
    }
    if c + 1 < width {
        arr[n] = (r, c + 1);
        n += 1;
    }

    arr.into_iter().take(n)
}

fn flood_fill(
    cells: &[Vec<bool>],
    labels: &mut [Vec<usize>],
    start: (usize, usize),
    label: usize,
    height: usize,
    width: usize,
) -> Vec<(usize, usize)> {
    let mut stack = vec![start];
    let mut component = Vec::new();
    labels[start.0][start.1] = label;

    while let Some((r, c)) = stack.pop() {
        component.push((r, c));

        for (nr, nc) in neighbors_4(r, c, height, width) {
            if cells[nr][nc] && labels[nr][nc] == 0 {
                labels[nr][nc] = label;
                stack.push((nr, nc));
            }
        }
    }

    component
}

fn remove_noise(
    mut cells: Vec<Vec<bool>>,
    width: usize,
    height: usize,
    min_size: usize,
) -> Vec<Vec<bool>> {
    let mut labels = vec![vec![0usize; width]; height];
    let mut label = 0usize;
    let mut components: Vec<Vec<(usize, usize)>> = Vec::new();

    for row in 0..height {
        for col in 0..width {
            if !cells[row][col] || labels[row][col] != 0 {
                continue;
            }
            label += 1;
            components.push(flood_fill(
                &cells,
                &mut labels,
                (row, col),
                label,
                height,
                width,
            ));
        }
    }

    for component in components {
        if component.len() < min_size {
            for (r, c) in component {
                cells[r][c] = false;
            }
        }
    }

    cells
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, GrayImage, ImageFormat};

    use super::*;
    use crate::types::Cell;

    fn encode_png(img: DynamicImage) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut bytes);
        img.write_to(&mut cursor, ImageFormat::Png).unwrap();
        bytes
    }

    #[test]
    fn image_to_grid_converts_quadrants() {
        let mut img = GrayImage::from_pixel(4, 4, Luma([255]));
        img.put_pixel(0, 0, Luma([0]));
        img.put_pixel(1, 0, Luma([0]));
        img.put_pixel(0, 1, Luma([0]));
        img.put_pixel(1, 1, Luma([0]));

        let bytes = encode_png(DynamicImage::ImageLuma8(img));
        let grid = image_to_grid(
            &bytes,
            &ImageConvertParams {
                grid_width: 2,
                grid_height: 2,
                smooth_strength: 0.0,
                threshold: 128,
                edge_strength: 0.0,
                noise_removal: 0,
            },
        )
        .unwrap();

        assert_eq!(grid.width(), 2);
        assert_eq!(grid.height(), 2);
        assert_eq!(grid.cell(0, 0), &Cell::Filled);
        assert_eq!(grid.cell(0, 1), &Cell::Blank);
        assert_eq!(grid.cell(1, 0), &Cell::Blank);
        assert_eq!(grid.cell(1, 1), &Cell::Blank);
    }

    #[test]
    fn remove_noise_removes_small_component() {
        let cells = vec![
            vec![true, false, false],
            vec![false, true, true],
            vec![false, false, false],
        ];

        let cleaned = remove_noise(cells, 3, 3, 2);
        assert!(!cleaned[0][0]);
        assert!(cleaned[1][1]);
        assert!(cleaned[1][2]);
    }
}
