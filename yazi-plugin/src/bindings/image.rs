use std::{fmt::Write, ops::Deref};

use mlua::{MetaMethod, UserData};
use resvg::usvg;

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

pub struct SvgInfo(yazi_adapter::SvgInfo);

impl Deref for SvgInfo {
	type Target = yazi_adapter::SvgInfo;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<yazi_adapter::SvgInfo> for SvgInfo {
	fn from(value: yazi_adapter::SvgInfo) -> Self { Self(value) }
}

impl UserData for SvgInfo {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("w", |_, this| Ok(this.width));
		fields.add_field_method_get("h", |_, this| Ok(this.height));

		fields.add_field_method_get("layers", |lua, this| {
			let table = lua.create_table()?;
			for (i, layer) in this.layers.iter().enumerate() {
				table.set(i + 1, layer.clone())?;
			}
			Ok(table)
		});

		fields.add_field_method_get("fonts", |lua, this| {
			let table = lua.create_table()?;
			for (i, font) in this.fonts.iter().enumerate() {
				table.set(i + 1, Font(font.clone()))?;
			}
			Ok(table)
		});
	}
}

pub struct Font(usvg::Font);

impl Deref for Font {
	type Target = usvg::Font;

	fn deref(&self) -> &Self::Target { &self.0 }
}

trait ToStaticStr {
	fn as_str(&self) -> &'static str;
}

impl ToStaticStr for usvg::FontStyle {
	fn as_str(&self) -> &'static str {
		match self {
			Self::Normal => "Normal",
			Self::Italic => "Italic",
			Self::Oblique => "Oblique",
		}
	}
}

impl ToStaticStr for usvg::FontStretch {
	fn as_str(&self) -> &'static str {
		match self {
			Self::UltraCondensed => "UltraCondensed",
			Self::ExtraCondensed => "ExtraCondensed",
			Self::Condensed => "Condensed",
			Self::SemiCondensed => "SemiCondensed",
			Self::Normal => "Normal",
			Self::SemiExpanded => "SemiExpanded",
			Self::Expanded => "Expanded",
			Self::ExtraExpanded => "ExtraExpanded",
			Self::UltraExpanded => "UltraExpanded",
		}
	}
}

impl UserData for Font {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("families", |_, this| {
			let mut families = String::new();
			for (i, family) in this.families().iter().enumerate() {
				if i > 0 {
					families.push_str(", ");
				}
				write!(&mut families, "{}", family).unwrap();
			}
			Ok(families)
		});
		fields.add_field_method_get("style", |_, this| Ok(this.style().as_str()));
		fields.add_field_method_get("stretch", |_, this| Ok(this.stretch().as_str()));

		fields.add_field_method_get("weight", |_, this| Ok(this.weight()));
	}

	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
			let mut families = String::new();
			for (i, family) in this.families().iter().enumerate() {
				if i > 0 {
					families.push_str(", ");
				}
				write!(&mut families, "{}", family).unwrap();
			}

			Ok(format!(
				"{}, style: {}, stretch: {}, weight: {}",
				families,
				this.style().as_str(),
				this.stretch().as_str(),
				this.weight()
			))
		});
	}
}
