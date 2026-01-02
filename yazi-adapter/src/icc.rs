use anyhow::Context;
use image::{ColorType, DynamicImage, GrayAlphaImage, GrayImage, ImageDecoder, RgbImage, RgbaImage, metadata::Cicp};
use moxcms::{CicpColorPrimaries, ColorProfile, DataColorSpace, Layout, TransferCharacteristics, TransformOptions};

pub(super) struct Icc;
impl Icc {
	pub(super) fn transform(mut decoder: impl ImageDecoder) -> anyhow::Result<DynamicImage> {
		if let Some(layout) = Self::color_type_to_layout(decoder.color_type())
			&& let Some(icc) = decoder.icc_profile().unwrap_or_default()
			&& let Ok(profile) = ColorProfile::new_from_slice(&icc)
			&& Self::requires_transform(&profile)
		{
			let mut buf = vec![0u8; decoder.total_bytes() as usize];
			let (w, h) = decoder.dimensions();
			decoder.read_image(&mut buf)?;

			let transformer = profile
				// TODO: Use `create_transform_in_place_nbit` in the next minor version of moxcms.
				.create_transform_8bit(layout, &ColorProfile::new_srgb(), layout, TransformOptions::default())
				.context("cannot make a profile transformer")?;

			let mut converted = vec![0u8; buf.len()];
			transformer.transform(&buf, &mut converted).context("cannot transform image")?;

			let mut image: DynamicImage = match layout {
				Layout::Gray => {
					GrayImage::from_raw(w, h, converted).context("cannot load transformed image")?.into()
				}
				Layout::GrayAlpha => {
					GrayAlphaImage::from_raw(w, h, converted).context("cannot load transformed image")?.into()
				}
				Layout::Rgb => {
					RgbImage::from_raw(w, h, converted).context("cannot load transformed image")?.into()
				}
				Layout::Rgba => {
					RgbaImage::from_raw(w, h, converted).context("cannot load transformed image")?.into()
				}
				_ => unreachable!(),
			};

			image.set_rgb_primaries(Cicp::SRGB.primaries);
			image.set_transfer_function(Cicp::SRGB.transfer);
			Ok(image)
		} else {
			Ok(DynamicImage::from_decoder(decoder)?)
		}
	}

	fn color_type_to_layout(color_type: ColorType) -> Option<Layout> {
		match color_type {
			ColorType::L8 => Some(Layout::Gray),
			ColorType::La8 => Some(Layout::GrayAlpha),
			ColorType::Rgb8 => Some(Layout::Rgb),
			ColorType::Rgba8 => Some(Layout::Rgba),
			_ => None,
		}
	}

	fn requires_transform(profile: &ColorProfile) -> bool {
		if profile.color_space == DataColorSpace::Cmyk {
			return false;
		}

		profile.cicp.is_none_or(|c| {
			c.color_primaries != CicpColorPrimaries::Bt709
				|| c.transfer_characteristics != TransferCharacteristics::Srgb
		})
	}
}
