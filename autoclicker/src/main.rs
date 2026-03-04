mod cg;

use cg::{CGMouseButton, click, get_mouse_position};
use std::time::Duration;

fn main() {
    let interval = Duration::from_millis(500);

    println!("Autoclicker starting in 2 seconds... (Ctrl+C to stop)");
    std::thread::sleep(Duration::from_secs(2));

    println!("Clicking every {}ms!", interval.as_millis());

    loop {
        let pos = get_mouse_position();
        click(pos, CGMouseButton::Right);
        std::thread::sleep(interval);
    }
}
