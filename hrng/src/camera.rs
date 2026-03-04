// FFI contract with csrc/camera.swift:
//   camera_init()       -> 0 success, -1 error
//   camera_get_frame()  -> >0 bytes needed (copied if buf fit), 0 no frame yet, <0 error
//   camera_stop()       -> void

unsafe extern "C" {
    fn camera_init() -> i32;
    fn camera_get_frame(buf: *mut u8, buf_len: i32, out_width: *mut i32, out_height: *mut i32) -> i32;
    fn camera_stop();
}

pub struct Camera;

pub struct FrameInfo {
    pub nbytes: usize,
    pub width: usize,
    pub height: usize,
}

impl Camera {
    pub fn open() -> Result<Self, &'static str> {
        let rc = unsafe { camera_init() };
        if rc != 0 {
            return Err("camera_init failed (check camera permission)");
        }
        Ok(Camera)
    }

    /// Grab the latest BGRA frame into `buf`.
    /// Returns Ok(None) when the camera hasn't produced a frame yet.
    /// Tries at most twice: once with current buffer, once after resize.
    pub fn grab_frame(&self, buf: &mut Vec<u8>) -> Result<Option<FrameInfo>, &'static str> {
        for _ in 0..2 {
            let mut w: i32 = 0;
            let mut h: i32 = 0;

            // Safe truncation: camera frames are well under 2 GB.
            let buf_len = i32::try_from(buf.len()).unwrap_or(i32::MAX);
            let ptr = if buf.is_empty() { std::ptr::null_mut() } else { buf.as_mut_ptr() };

            let rc = unsafe { camera_get_frame(ptr, buf_len, &mut w, &mut h) };

            if rc < 0 { return Err("camera_get_frame failed"); }
            if rc == 0 { return Ok(None); }

            let needed = rc as usize;
            if needed <= buf.len() {
                return Ok(Some(FrameInfo {
                    nbytes: needed,
                    width: w.max(0) as usize,
                    height: h.max(0) as usize,
                }));
            }

            // Buffer too small - resize and retry once.
            buf.resize(needed, 0);
        }
        Err("frame size keeps changing")
    }
}

impl Drop for Camera {
    fn drop(&mut self) {
        unsafe { camera_stop() };
    }
}
