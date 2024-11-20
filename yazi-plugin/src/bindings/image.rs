use std::ops::Deref;

use mlua::{MetaMethod, UserData};

pub struct ImageInfo(yazi_adapter::ImageInfo);

impl Deref for ImageInfo {
	type Target = yazi_adapter::ImageInfo;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<yazi_adapter::ImageInfo> for ImageInfo {
	fn from(value: yazi_adapter::ImageInfo) -> Self { Self(value) }
}

impl UserData for ImageInfo {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("w", |_, me| Ok(me.width));
		fields.add_field_method_get("h", |_, me| Ok(me.height));
		fields.add_field_method_get("ori", |_, me| Ok(me.orientation.map(|o| o.to_exif())));
		fields.add_field_method_get("format", |_, me| Ok(ImageFormat(me.format)));
		fields.add_field_method_get("color", |_, me| Ok(ImageColor(me.color)));
	}
}

// --- ImageFormat
struct ImageFormat(yazi_adapter::ImageFormat);

impl UserData for ImageFormat {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| {
			use yazi_adapter::ImageFormat as F;

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
				F::Pcx => "PCX",
				_ => "Unknown",
			})
		});
	}
}

// --- ImageColor
struct ImageColor(yazi_adapter::ImageColor);

impl UserData for ImageColor {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| {
			use yazi_adapter::ImageColor as C;

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
