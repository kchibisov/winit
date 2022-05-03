use std::sync::Arc;
use std::{error, fmt};

pub type MimePicker = Arc<dyn FnOnce(&[MimeType]) -> MimeType>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MimeType {
    /// UTF-8 text.
    Text,

    /// Raw image data.
    RawImage,

    /// Png image.
    PngImage,
}

#[derive(Debug, Clone)]
pub enum ClipboardMimedContent {
    Text(String),

    RawImage(RawImage),

    PngImage(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct RawImage {
    width: usize,

    height: usize,

    buffer: Vec<u8>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    /// TODO.
    Failed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Clipboard operation failed.")
    }
}

impl error::Error for Error {}
