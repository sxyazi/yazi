use std::path::Path;

use image::{ImageDecoder, ImageError};

pub type ImageFormat = image::ImageFormat;
pub type ImageColor = image::ColorType;
pub type ImageOrientation = image::metadata::Orientation;

#[derive(Clone, Copy)]
pub struct ImageInfo {
	pub format:      ImageFormat,
	pub width:       u32,
	pub height:      u32,
	pub color:       ImageColor,
	pub orientation: Option<ImageOrientation>,
}

impl ImageInfo {
	pub async fn new(path: &Path) -> image::ImageResult<Self> {
		let path = path.to_owned();
		tokio::task::spawn_blocking(move || {
			let reader = image::ImageReader::open(path)?.with_guessed_format()?;

			let Some(format) = reader.format() else {
				return Err(ImageError::IoError(std::io::Error::new(
					std::io::ErrorKind::InvalidData,
					"unknown image format",
				)));
			};

			let mut decoder = reader.into_decoder()?;
			let (width, height) = decoder.dimensions();
			Ok(Self {
				format,
				width,
				height,
				color: decoder.color_type(),
				orientation: decoder.orientation().ok(),
			})
		})
		.await
		.map_err(|e| ImageError::IoError(e.into()))?
	}
}
