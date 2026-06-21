use std::path::PathBuf;

use image::{ImageDecoder, ImageError};
use mlua::{MetaMethod, UserData, UserDataFields, UserDataMethods};

// --- ImageInfo
#[derive(Clone, Copy)]
pub struct ImageInfo {
	pub format:      image::ImageFormat,
	pub width:       u32,
	pub height:      u32,
	pub color:       image::ColorType,
	pub orientation: Option<image::metadata::Orientation>,
}

impl ImageInfo {
	pub async fn new(path: PathBuf) -> image::ImageResult<Self> {
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

impl UserData for ImageInfo {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("w", |_, me| Ok(me.width));
		fields.add_field_method_get("h", |_, me| Ok(me.height));
		fields.add_field_method_get("ori", |_, me| Ok(me.orientation.map(|o| o.to_exif())));
		fields.add_field_method_get("format", |_, me| Ok(ImageFormat(me.format)));
		fields.add_field_method_get("color", |_, me| Ok(ImageColor(me.color)));
	}
}

// --- ImageFormat
struct ImageFormat(image::ImageFormat);

impl UserData for ImageFormat {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| {
			use image::ImageFormat as F;

			Ok(match me.0 {
				F::Png => "PNG",
				F::Jpeg => "JPEG",
				F::Gif => "GIF",
				F::WebP => "WEBP",
				F::Pnm => "PNM",
				F::Tiff => "TIFF",
				F::Tga => "TGA",
				F::Dds => "DDS",
				F::Bmp => "BMP",
				F::Ico => "ICO",
				F::Hdr => "HDR",
				F::OpenExr => "OpenEXR",
				F::Farbfeld => "Farbfeld",
				F::Avif => "AVIF",
				F::Qoi => "QOI",
				_ => "Unknown",
			})
		});
	}
}

// --- ImageColor
struct ImageColor(image::ColorType);

impl UserData for ImageColor {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| {
			use image::ColorType as C;

			Ok(match me.0 {
				C::L8 => "L8",
				C::La8 => "La8",
				C::Rgb8 => "Rgb8",
				C::Rgba8 => "Rgba8",

				C::L16 => "L16",
				C::La16 => "La16",
				C::Rgb16 => "Rgb16",
				C::Rgba16 => "Rgba16",

				C::Rgb32F => "Rgb32F",
				C::Rgba32F => "Rgba32F",
				_ => "Unknown",
			})
		});
	}
}
