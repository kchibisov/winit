use std::{error, fmt};

pub enum MimeType {
    /// UTF-8 text.
    Text,

    /// Raw image data.
    RawImage,
}

pub enum ClipboardMimedContent {
    Text(String),

    RawImage(),
}

pub struct RawImage {
    width: usize,
    height: usize,
}

#[derive(Debug)]
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
