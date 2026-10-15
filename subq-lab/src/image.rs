use std::fmt;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::ser::{Serialize, SerializeStruct, Serializer};

use crate::{Result, SubqError};

const MIB: usize = 1024 * 1024;

pub(crate) const MAX_IMAGE_BYTES: usize = 4 * MIB;
pub(crate) const MAX_TOTAL_BYTES: usize = 8 * MIB;
pub(crate) const MAX_IMAGE_COUNT: usize = 4;

/// Image formats the Decisions API accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MediaType {
    Png,
    Jpeg,
    Webp,
}

impl MediaType {
    /// Detects the format from the file's first bytes ("magic numbers"),
    /// so a PNG renamed to `.jpg` is still labelled correctly.
    pub fn sniff(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some(Self::Png),
            [0xFF, 0xD8, 0xFF, ..] => Some(Self::Jpeg),
            b if b.starts_with(b"RIFF") && b.get(8..12) == Some(b"WEBP") => Some(Self::Webp),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }
}

/// An image to send with a decision. Holds the raw bytes; base64 happens
/// only when the request is serialized.
#[derive(Clone)]
pub struct Image {
    media_type: MediaType,
    bytes: Vec<u8>,
}

impl Image {
    /// Checks the format (PNG, JPEG, or WebP) and the 4 MiB limit.
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Result<Self> {
        let bytes = bytes.into();
        let media_type = MediaType::sniff(&bytes)
            .ok_or_else(|| SubqError::Invalid("image must be PNG, JPEG, or WebP".to_string()))?;
        if bytes.len() > MAX_IMAGE_BYTES {
            return Err(SubqError::Invalid(format!(
                "image is {} bytes; the limit is 4 MiB",
                bytes.len()
            )));
        }
        Ok(Self { media_type, bytes })
    }

    /// Reads the file with blocking I/O. In async code with many files,
    /// read them with `tokio::fs::read` and call [`Image::from_bytes`].
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_bytes(std::fs::read(path)?)
    }

    pub fn media_type(&self) -> MediaType {
        self.media_type
    }

    pub fn bytes_len(&self) -> usize {
        self.bytes.len()
    }
}

// Serializes as `{ "mediaType": "image/png", "base64": "iVBOR..." }`.
impl Serialize for Image {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Image", 2)?;
        s.serialize_field("mediaType", self.media_type.as_str())?;
        s.serialize_field("base64", &STANDARD.encode(&self.bytes))?;
        s.end()
    }
}

// Don't dump megabytes of bytes into logs.
impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image")
            .field("media_type", &self.media_type)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const PNG_HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

    #[test]
    fn sniffs_formats() {
        assert_eq!(MediaType::sniff(&PNG_HEADER), Some(MediaType::Png));
        assert_eq!(
            MediaType::sniff(&[0xFF, 0xD8, 0xFF, 0xE0]),
            Some(MediaType::Jpeg)
        );
        assert_eq!(
            MediaType::sniff(b"RIFF\0\0\0\0WEBPVP8 "),
            Some(MediaType::Webp)
        );
        assert_eq!(MediaType::sniff(b"GIF89a"), None);
        assert_eq!(MediaType::sniff(&[]), None);
        assert_eq!(MediaType::sniff(b"RIFF\0\0\0\0WAVEfmt "), None);
    }

    #[test]
    fn serializes_like_the_docs() {
        let img = Image::from_bytes(PNG_HEADER).unwrap();
        assert_eq!(
            serde_json::to_value(&img).unwrap(),
            json!({ "mediaType": "image/png", "base64": "iVBORw0KGgo=" })
        );
    }

    #[test]
    fn rejects_oversized_and_unknown() {
        let mut big = PNG_HEADER.to_vec();
        big.resize(MAX_IMAGE_BYTES + 1, 0);
        assert!(Image::from_bytes(big).is_err());
        assert!(Image::from_bytes(b"not an image".to_vec()).is_err());
    }
}
