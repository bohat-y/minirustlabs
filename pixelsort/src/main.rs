use image::{Rgb, RgbImage};

const LOW: u32 = 60;
const HIGH: u32 = 200;

fn brightness(pixel: &[u8; 3]) -> u32 {
    let [r, g, b] = *pixel;
    (r as u32 + g as u32 + b as u32) / 3
}

/// Sorts one line of pixels (a row or a column) section by section.
fn sort_line(pixels: &mut [[u8; 3]]) {
    let mut start: Option<usize> = None;
    for i in 0..pixels.len() {
        let inside = (LOW..=HIGH).contains(&brightness(&pixels[i]));

        if inside && start.is_none() {
            start = Some(i);
        } else if !inside {
            if let Some(s) = start {
                pixels[s..i].sort_by_key(brightness);
                // pixels[s..i].reverse();
                start = None;
            }
        }
    }

    if let Some(s) = start {
        pixels[s..].sort_by_key(brightness);
    }
    // for chunk in pixels.chunks_mut(40) {
    //     chunk.sort_by_key(brightness);
    // }
}

/// Rows are contiguous in memory, so each one can be sorted in place.
fn sort_horizontal(img: &mut RgbImage) {
    let w = img.width() as usize;
    for row in img.chunks_exact_mut(w * 3) {
        let (pixels, _) = row.as_chunks_mut::<3>();
        sort_line(pixels);
    }
}

/// Columns are spread out in memory, so copy each one into a Vec,
/// sort it, and write it back.
fn sort_vertical(img: &mut RgbImage) {
    let (w, h) = img.dimensions();
    let mut column: Vec<[u8; 3]> = Vec::with_capacity(h as usize);
    for x in 0..w {
        column.clear();
        for y in 0..h {
            column.push(img.get_pixel(x, y).0);
        }

        sort_line(&mut column);

        for y in 0..h {
            img.put_pixel(x, y, Rgb(column[y as usize]));
        }
    }
}

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/mona-lisa.jpeg");
    let original = image::open(path).unwrap().to_rgb8();
    println!("{}x{}", original.width(), original.height());

    let mut horizontal = original.clone();
    sort_horizontal(&mut horizontal);
    horizontal
        .save(concat!(env!("CARGO_MANIFEST_DIR"), "/out_horizontal.png"))
        .unwrap();

    let mut vertical = original.clone();
    sort_vertical(&mut vertical);
    vertical
        .save(concat!(env!("CARGO_MANIFEST_DIR"), "/out_vertical.png"))
        .unwrap();
}
