mod camera;
mod rng;

use camera::Camera;
use minifb::{Key, Window, WindowOptions};
use rng::EntropyPool;

const WIDTH: usize = 640;
const HEIGHT: usize = 350;

fn main() {
    let cam = Camera::open().expect("Failed to open camera");
    let mut window = Window::new("HRNG - Demo", WIDTH, HEIGHT, WindowOptions::default())
        .expect("Failed to create window");

    window.set_target_fps(30);

    let mut bgra_buf = Vec::<u8>::new();
    let mut framebuf = vec![0u32; WIDTH * HEIGHT];
    let mut idle_ticks: u32 = 0;
    let mut pool = EntropyPool::new();
    let mut random_out = [0u8; 32];
    let mut frame_nr: u64 = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame = match cam.grab_frame(&mut bgra_buf) {
            Ok(Some(f)) if f.width > 0 && f.height > 0 => {
                idle_ticks = 0;
                f
            }
            Ok(_) => {
                idle_ticks = idle_ticks.saturating_add(1);
                if idle_ticks % 200 == 0 {
                    eprintln!("waiting for camera frames...");
                }
                window.update_with_buffer(&framebuf, WIDTH, HEIGHT).ok();
                continue;
            }
            Err(e) => {
                eprintln!("camera error: {e}");
                break;
            }
        };

        let (sw, sh) = (frame.width, frame.height);

        if frame.nbytes < sw * sh * 4 {
            continue;
        }

        frame_nr += 1;
        let frame_data = &bgra_buf[..frame.nbytes];

        pool.feed_camera(frame_data);
        pool.feed_timing();

        // Nearest-neighbor scale BGRA -> 0x00RRGGBB for minifb.
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let off = (y * sh / HEIGHT * sw + x * sw / WIDTH) * 4;
                let (b, g, r) = (bgra_buf[off] as u32, bgra_buf[off + 1] as u32, bgra_buf[off + 2] as u32);
                framebuf[y * WIDTH + x] = (r << 16) | (g << 8) | b;
            }
        }

        if pool.generate(&mut random_out) {
            eprint!("rng: ");
            for b in &random_out { eprint!("{:02x}", b); }
            eprintln!();
        }

        window.update_with_buffer(&framebuf, WIDTH, HEIGHT).ok();
    }
}
