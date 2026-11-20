use image::RgbImage;

/// What the pixels inside a section are ordered by.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SortKey {
    Brightness,
    Hue,
    Saturation,
    Red,
    Green,
    Blue,
}

impl SortKey {
    pub const ALL: [SortKey; 6] = [
        SortKey::Brightness,
        SortKey::Hue,
        SortKey::Saturation,
        SortKey::Red,
        SortKey::Green,
        SortKey::Blue,
    ];

    pub fn value(self, pixel: &[u8; 3]) -> u32 {
        match self {
            SortKey::Brightness => brightness(pixel),
            SortKey::Hue => hue(pixel),
            SortKey::Saturation => saturation(pixel),
            SortKey::Red => pixel[0] as u32,
            SortKey::Green => pixel[1] as u32,
            SortKey::Blue => pixel[2] as u32,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Settings {
    /// Direction of the lines: 0° = left to right, 90° = top to bottom.
    pub angle: f32,
    pub low: u32,
    pub high: u32,
    pub key: SortKey,
    pub reverse: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            angle: 0.0,
            low: 60,
            high: 200,
            key: SortKey::Brightness,
            reverse: false,
        }
    }
}

/// 0 (black) to 255 (white)
pub fn brightness(pixel: &[u8; 3]) -> u32 {
    let [r, g, b] = *pixel;
    (r as u32 + g as u32 + b as u32) / 3
}

/// Position on the color wheel: 0 = red, 120 = green, 240 = blue (0..360)
fn hue(pixel: &[u8; 3]) -> u32 {
    let [r, g, b] = pixel.map(|c| c as i32);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    if d == 0 {
        return 0; // gray has no hue
    }
    let h = if max == r {
        60 * (g - b) / d
    } else if max == g {
        60 * (b - r) / d + 120
    } else {
        60 * (r - g) / d + 240
    };
    h.rem_euclid(360) as u32
}

/// 0 (gray) to 255 (pure color)
fn saturation(pixel: &[u8; 3]) -> u32 {
    let max = *pixel.iter().max().unwrap() as u32;
    let min = *pixel.iter().min().unwrap() as u32;
    if max == 0 { 0 } else { (max - min) * 255 / max }
}

/// Sorts one line of pixels section by section. A section is a run of
/// pixels whose brightness is between `low` and `high`.
fn sort_line(pixels: &mut [[u8; 3]], s: &Settings) {
    let mut start: Option<usize> = None;
    for i in 0..pixels.len() {
        let inside = (s.low..=s.high).contains(&brightness(&pixels[i]));

        if inside && start.is_none() {
            start = Some(i);
        } else if !inside {
            if let Some(st) = start {
                sort_section(&mut pixels[st..i], s);
                start = None;
            }
        }
    }

    if let Some(st) = start {
        sort_section(&mut pixels[st..], s);
    }
}

fn sort_section(section: &mut [[u8; 3]], s: &Settings) {
    section.sort_by_key(|p| s.key.value(p));
    if s.reverse {
        section.reverse();
    }
}

/// Splits the image into lines running at `angle` degrees. Each line is a
/// list of pixel indices, in order along the line.
///
/// Every pixel gets two coordinates in a rotated frame: `across` says which
/// line it is on, `along` says where on that line it sits. At 0° that's just
/// (y, x), i.e. rows; at 90° it's (-x, y), i.e. columns.
pub fn lines_at_angle(width: u32, height: u32, angle: f32) -> Vec<Vec<usize>> {
    let (sin, cos) = angle.to_radians().sin_cos();

    let mut tagged: Vec<(i32, f32, usize)> = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            let (xf, yf) = (x as f32, y as f32);
            let across = (yf * cos - xf * sin).round() as i32;
            let along = xf * cos + yf * sin;
            tagged.push((across, along, (y * width + x) as usize));
        }
    }

    tagged.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    tagged
        .chunk_by(|a, b| a.0 == b.0)
        .map(|line| line.iter().map(|t| t.2).collect())
        .collect()
}

/// Lines aren't contiguous in memory (except at 0°), so copy each one into
/// a buffer, sort it, and write it back - same idea as the old sort_vertical.
pub fn sort_image(img: &mut RgbImage, lines: &[Vec<usize>], s: &Settings) {
    let (pixels, _) = img.as_chunks_mut::<3>();
    let mut buf: Vec<[u8; 3]> = Vec::new();
    for line in lines {
        buf.clear();
        buf.extend(line.iter().map(|&i| pixels[i]));

        sort_line(&mut buf, s);

        for (&i, &p) in line.iter().zip(&buf) {
            pixels[i] = p;
        }
    }
}
