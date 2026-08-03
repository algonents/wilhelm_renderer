use image::ImageReader;
use std::io::Cursor;

// core/image.rs
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA8 format
}

/// Where image bytes come from. Every source converges on a byte slice
/// before decoding; only the bytes ever reach the decoder, so backends
/// without a filesystem (wasm) use `Bytes` (e.g. via `include_bytes!`
/// or bytes fetched by the host page).
pub enum ImageSource<'a> {
    Path(&'a str),
    Bytes(&'a [u8]),
}

impl<'a> From<&'a str> for ImageSource<'a> {
    fn from(path: &'a str) -> Self {
        ImageSource::Path(path)
    }
}

impl<'a> From<&'a [u8]> for ImageSource<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        ImageSource::Bytes(bytes)
    }
}

// `include_bytes!` yields `&[u8; N]`, which does not coerce through
// a generic `impl Into<ImageSource>` parameter.
impl<'a, const N: usize> From<&'a [u8; N]> for ImageSource<'a> {
    fn from(bytes: &'a [u8; N]) -> Self {
        ImageSource::Bytes(bytes)
    }
}

#[derive(Debug)]
pub enum ImageError {
    Io(std::io::Error),
    Decode(image::ImageError),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::Io(e) => write!(f, "failed to read image: {e}"),
            ImageError::Decode(e) => write!(f, "failed to decode image: {e}"),
        }
    }
}

impl std::error::Error for ImageError {}

pub fn try_load_image<'a>(source: impl Into<ImageSource<'a>>) -> Result<Image, ImageError> {
    let bytes = match source.into() {
        ImageSource::Path(path) => std::fs::read(path).map_err(ImageError::Io)?,
        ImageSource::Bytes(bytes) => bytes.to_vec(),
    };

    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(ImageError::Io)?
        .decode()
        .map_err(ImageError::Decode)?
        .to_rgba8();

    let (width, height) = img.dimensions();
    let pixels = img.into_raw();

    Ok(Image {
        width,
        height,
        pixels,
    })
}

pub fn load_image<'a>(source: impl Into<ImageSource<'a>>) -> Image {
    try_load_image(source).unwrap_or_else(|e| panic!("{e}"))
}
