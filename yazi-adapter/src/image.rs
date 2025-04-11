use std::{path::{Path, PathBuf}, sync::{LazyLock, RwLock}};

use anyhow::Result;
use image::{DynamicImage, ExtendedColorType, ImageBuffer, ImageDecoder, ImageEncoder, ImageError, ImageReader, ImageResult, Limits, codecs::{jpeg::JpegEncoder, png::PngEncoder}, error::UnsupportedErrorKind, imageops::FilterType, metadata::Orientation};
use ratatui::layout::Rect;
use resvg::{tiny_skia::{Pixmap, Transform}, usvg::{Options, Tree}};
use yazi_config::YAZI;

use crate::Dimension;

pub static GLOBAL_OPTIONS: LazyLock<RwLock<Options<'static>>> =
	LazyLock::new(|| RwLock::new(Options::default()));

pub struct Image;

impl Image {
	pub async fn precache(path: &Path, cache: PathBuf) -> Result<()> {
		let (img, icc) =
			Self::decode_to_fit_from(path, YAZI.preview.max_width, YAZI.preview.max_height).await?;

		let buf = tokio::task::spawn_blocking(move || {
			let mut buf = Vec::new();
			if img.color().has_alpha() {
				let rgba = img.into_rgba8();
				let mut encoder = PngEncoder::new(&mut buf);
				icc.map(|b| encoder.set_icc_profile(b));
				encoder.write_image(&rgba, rgba.width(), rgba.height(), ExtendedColorType::Rgba8)?;
			} else {
				let mut encoder = JpegEncoder::new_with_quality(&mut buf, YAZI.preview.image_quality);
				icc.map(|b| encoder.set_icc_profile(b));
				encoder.encode_image(&img.into_rgb8())?;
			}

			Ok::<_, ImageError>(buf)
		})
		.await??;

		Ok(tokio::fs::write(cache, buf).await?)
	}

	pub(super) async fn downscale(path: &Path, rect: Rect) -> Result<DynamicImage> {
		let (width, height) = Self::max_pixel(rect);
		let (img, _) = Self::decode_to_fit_from(path, width, height).await?;
		Ok(img)
	}

	pub(super) fn max_pixel(rect: Rect) -> (u32, u32) {
		Dimension::ratio()
			.map(|(r1, r2)| {
				let (w, h) = ((rect.width as f64 * r1) as u32, (rect.height as f64 * r2) as u32);
				(w.min(YAZI.preview.max_width), h.min(YAZI.preview.max_height))
			})
			.unwrap_or((YAZI.preview.max_width, YAZI.preview.max_height))
	}

	pub(super) fn pixel_area(size: (u32, u32), rect: Rect) -> Rect {
		Dimension::ratio()
			.map(|(r1, r2)| Rect {
				x:      rect.x,
				y:      rect.y,
				width:  (size.0 as f64 / r1).ceil() as u16,
				height: (size.1 as f64 / r2).ceil() as u16,
			})
			.unwrap_or(rect)
	}

	#[inline]
	fn filter() -> FilterType {
		match YAZI.preview.image_filter.as_str() {
			"nearest" => FilterType::Nearest,
			"triangle" => FilterType::Triangle,
			"catmull-rom" => FilterType::CatmullRom,
			"gaussian" => FilterType::Gaussian,
			"lanczos3" => FilterType::Lanczos3,
			_ => FilterType::Triangle,
		}
	}

	async fn decode_to_fit_from(
		path: &Path,
		width: u32,
		height: u32,
	) -> Result<(DynamicImage, Option<Vec<u8>>)> {
		let path = path.to_owned();

		tokio::task::spawn_blocking(move || {
			Self::try_decode_raster(&path, width, height).or_else(|err| match err {
				ImageError::Unsupported(ref unsupported) => match unsupported.kind() {
					UnsupportedErrorKind::Format(_) => Self::try_decode_svg(&path, width, height),
					_ => Err(err.into()),
				},
				_ => Err(err.into()),
			})
		})
		.await
		.map_err(|e| ImageError::IoError(e.into()))?
	}

	#[inline]
	fn try_decode_raster(
		path: &Path,
		width: u32,
		height: u32,
	) -> ImageResult<(DynamicImage, Option<Vec<u8>>)> {
		let limits = Self::build_limits();
		let mut reader = ImageReader::open(path)?;
		reader.limits(limits);
		let mut decoder = reader.with_guessed_format()?.into_decoder()?;
		let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
		let icc = decoder.icc_profile().unwrap_or_default();

		let mut img = DynamicImage::from_decoder(decoder)?;
		let (w, h) = Self::flip_size(orientation, (width, height));

		if img.width() > w || img.height() > h {
			img = img.resize(w, h, Self::filter());
		}
		if orientation != Orientation::NoTransforms {
			img.apply_orientation(orientation);
		}

		Ok((img, icc))
	}

	#[inline]
	fn build_limits() -> Limits {
		let mut limits = Limits::no_limits();
		if YAZI.tasks.image_alloc > 0 {
			limits.max_alloc = Some(YAZI.tasks.image_alloc as u64);
		}
		if YAZI.tasks.image_bound[0] > 0 {
			limits.max_image_width = Some(YAZI.tasks.image_bound[0] as u32);
		}
		if YAZI.tasks.image_bound[1] > 0 {
			limits.max_image_height = Some(YAZI.tasks.image_bound[1] as u32);
		}
		limits
	}

	#[inline]
	fn try_decode_svg(
		path: &Path,
		width: u32,
		height: u32,
	) -> Result<(DynamicImage, Option<Vec<u8>>)> {
		let pixmap = Self::render_svg_to_fit_from(path, width, height)?;
		let (width, height) = (pixmap.width(), pixmap.height());
		let mut container = pixmap.take();

		for rgba in container.chunks_exact_mut(4) {
			let alpha = rgba[3];
			if alpha != 0xff {
				let pixel: &mut [u8; 4] = unsafe { rgba.try_into().unwrap_unchecked() };
				let a = alpha as f64 / 255.0;
				pixel[0] = (pixel[0] as f64 / a + 0.5) as u8;
				pixel[1] = (pixel[1] as f64 / a + 0.5) as u8;
				pixel[2] = (pixel[2] as f64 / a + 0.5) as u8;
			}
		}

		let img = ImageBuffer::from_raw(width, height, container)
			.ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;

		Ok((DynamicImage::ImageRgba8(img), None))
	}

	/// Helper function to rasterize an SVG to a pixmap fitting within max_width
	/// and max_height.
	fn render_svg_to_fit_from(path: &Path, max_width: u32, max_height: u32) -> Result<Pixmap> {
		let svg = std::fs::read(path)?;
		let options_guard =
			GLOBAL_OPTIONS.read().map_err(|e| anyhow::anyhow!("RwLock poisoned: {}", e))?;
		let tree = Tree::from_data(&svg, &options_guard)?;
		let (width, height, transform) = Self::svg_size_and_scale(&tree, max_width, max_height);

		let mut pixmap =
			Pixmap::new(width, height).ok_or_else(|| anyhow::anyhow!("Cannot create pixmap"))?;

		resvg::render(&tree, transform, &mut pixmap.as_mut());

		Ok(pixmap)
	}

	fn svg_size_and_scale(tree: &Tree, max_width: u32, max_height: u32) -> (u32, u32, Transform) {
		// It is Ok. The max_width and max_height could not be larger then monitor
		// dimensions which much less then f32::MAX
		let max_width = max_width as f32;
		let max_height = max_height as f32;
		let mut width = tree.size().width();
		let mut height = tree.size().height();

		for node in tree.root().children() {
			if let Some(bounding_box) = node.abs_layer_bounding_box() {
				width = width.max(bounding_box.width());
				height = height.max(bounding_box.height());
			}
		}
		if width <= max_width && height <= max_height {
			return (width.floor() as u32, height.floor() as u32, Transform::from_scale(1.0, 1.0));
		}
		let ratio = f32::min(max_width / width, max_height / height);
		(
			(width * ratio).floor() as u32,
			(height * ratio).floor() as u32,
			Transform::from_scale(ratio, ratio),
		)
	}

	fn flip_size(orientation: Orientation, (w, h): (u32, u32)) -> (u32, u32) {
		use image::metadata::Orientation::{Rotate90, Rotate90FlipH, Rotate270, Rotate270FlipH};
		match orientation {
			Rotate90 | Rotate270 | Rotate90FlipH | Rotate270FlipH => (h, w),
			_ => (w, h),
		}
	}
}
