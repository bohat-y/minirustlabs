// TODO: Audio entropy source

pub struct AudioSource;

impl AudioSource {
    pub fn open() -> Result<Self, &'static str> {
        todo!("implement CoreAudio capture via Swift FFI")
    }

    pub fn grab_samples(&self, _buf: &mut Vec<u8>) -> Result<Option<usize>, &'static str> {
        todo!("grab raw PCM samples")
    }
}

impl Drop for AudioSource {
    fn drop(&mut self) {
        // TODO: call audio_stop()
    }
}
