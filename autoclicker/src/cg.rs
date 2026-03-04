use std::ffi::c_void;

// --- Types ---

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}

#[repr(u32)]
pub enum CGEventType {
    LeftMouseDown = 1,
    LeftMouseUp = 2,
    RightMouseDown = 3,
    RightMouseUp = 4,
    MouseMoved = 5,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum CGMouseButton {
    Left = 0,
    Right = 1,
}

#[repr(u32)]
pub enum CGEventTapLocation {
    HID = 0,
}

// --- Raw FFI bindings (private) ---

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventCreate(source: *const c_void) -> *mut c_void;
    fn CGEventGetLocation(event: *const c_void) -> CGPoint;
    fn CGEventCreateMouseEvent(
        source: *const c_void,
        mouse_type: u32,
        mouse_cursor_position: CGPoint,
        mouse_button: u32,
    ) -> *mut c_void;
    fn CGEventPost(tap: u32, event: *const c_void);
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(cf: *const c_void);
}

// --- public wrappers  ---

pub fn get_mouse_position() -> CGPoint {
    unsafe {
        let event = CGEventCreate(std::ptr::null());
        let pos = CGEventGetLocation(event);
        CFRelease(event);
        pos
    }
}

pub fn click(pos: CGPoint, button: CGMouseButton) {
    let (down_type, up_type) = match button {
        CGMouseButton::Left => (CGEventType::LeftMouseDown, CGEventType::LeftMouseUp),
        CGMouseButton::Right => (CGEventType::RightMouseDown, CGEventType::RightMouseUp),
    };

    unsafe {
        let mouse_down = CGEventCreateMouseEvent(
            std::ptr::null(),
            down_type as u32,
            pos,
            button as u32,
        );
        CGEventPost(CGEventTapLocation::HID as u32, mouse_down);
        CFRelease(mouse_down);

        let mouse_up = CGEventCreateMouseEvent(
            std::ptr::null(),
            up_type as u32,
            pos,
            button as u32,
        );
        CGEventPost(CGEventTapLocation::HID as u32, mouse_up);
        CFRelease(mouse_up);
    }
}
