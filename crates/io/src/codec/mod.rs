//! Decoders for compressed AVI video, selected by the FourCC in the bitmap header.

use planets_core::Image;

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("corrupt {codec} frame: {reason}")]
    Corrupt { codec: &'static str, reason: String },
    #[error("unsupported {codec} variant: {reason}")]
    Unsupported { codec: &'static str, reason: String },
}

/// What a decoder knows about the stream besides the frame bytes.
#[derive(Debug, Clone, Copy)]
pub struct FrameFormat<'a> {
    pub width: u32,
    pub height: u32,
    /// `biBitCount` from the bitmap header.
    pub bits_per_pixel: u16,
    /// Bytes of the `strf` chunk after the bitmap header (codec private data, e.g. HuffYUV tables).
    pub extra: &'a [u8],
}

/// Decodes one frame to an image with samples in `0.0..=1.0`, top row first.
type Decoder = fn(&FrameFormat, &[u8]) -> Result<Image, CodecError>;

// Stays a match so each codec only adds arms; drop the allow once the first arm exists.
#[allow(clippy::match_single_binding)]
fn decoder(fourcc: &[u8; 4]) -> Option<Decoder> {
    // Each codec adds its FourCCs here (upper case).
    match &fourcc.to_ascii_uppercase() {
        _ => None,
    }
}

/// Whether a decoder exists for the FourCC (case-insensitive).
pub fn is_supported(fourcc: &[u8; 4]) -> bool {
    decoder(fourcc).is_some()
}

/// Decodes one frame; `fourcc` must be supported (see [`is_supported`]).
pub fn decode(fourcc: &[u8; 4], format: &FrameFormat, data: &[u8]) -> Result<Image, CodecError> {
    let decode = decoder(fourcc).ok_or_else(|| CodecError::Unsupported {
        codec: "AVI",
        reason: format!("no decoder for '{}'", String::from_utf8_lossy(fourcc)),
    })?;
    decode(format, data)
}
