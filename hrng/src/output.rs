// TODO: Random output formatting and delivery.
//
// Planned modes:
//   - Raw binary to stdout (pipe-friendly: ./hrng | xxd)
//   - Hex-encoded to stderr (current behavior, for debugging)
//   - Write to file
//   - Pipeline visualization table (show per-pixel transformation)

use std::io::{self, Write};

pub enum OutputMode {
    Hex,
    Raw,
}

pub struct Output {
    mode: OutputMode,
}

impl Output {
    pub fn new(mode: OutputMode) -> Self {
        Self { mode }
    }

    pub fn emit(&self, bytes: &[u8]) -> io::Result<()> {
        match self.mode {
            OutputMode::Hex => {
                let mut stderr = io::stderr().lock();
                write!(stderr, "rng: ")?;
                for b in bytes { write!(stderr, "{:02x}", b)?; }
                writeln!(stderr)?;
                Ok(())
            }
            OutputMode::Raw => {
                let mut stdout = io::stdout().lock();
                stdout.write_all(bytes)?;
                stdout.flush()
            }
        }
    }
}
