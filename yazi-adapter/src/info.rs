use std::{collections::HashSet, path::Path};

use image::{ImageDecoder, ImageError};
use resvg::usvg::{Font, Group, ImageKind, Node, Tree};

use super::image::GLOBAL_OPTIONS;

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

#[derive(Debug, Clone)]
pub struct SvgInfo {
	pub width:  f32,
	pub height: f32,
	pub fonts:  HashSet<Font>,
	pub layers: Vec<String>,
}

impl SvgInfo {
	pub async fn new(path: &Path) -> anyhow::Result<Self> {
		let path = path.to_owned();
		tokio::task::spawn_blocking(move || {
			let svg = std::fs::read(path)?;
			let options_guard =
				GLOBAL_OPTIONS.read().map_err(|e| anyhow::anyhow!("RwLock poisoned: {}", e))?;
			let tree = Tree::from_data(&svg, &options_guard)?;

			let mut info = SvgInfo {
				width:  tree.size().width().round(),
				height: tree.size().height().round(),
				fonts:  HashSet::new(),
				layers: Vec::new(),
			};

			Self::collect_group(tree.root(), &mut info);

			Ok(info)
		})
		.await?
	}

	fn collect_group(group: &Group, info: &mut SvgInfo) {
		if !group.id().is_empty() {
			info.layers.push(group.id().to_string());
		}

		for node in group.children() {
			Self::collect_node(node, info);
		}
	}

	fn collect_node(node: &Node, info: &mut SvgInfo) {
		if let Some(bounding_box) = node.abs_layer_bounding_box() {
			info.width = info.width.max(bounding_box.width());
			info.height = info.height.max(bounding_box.height());
		}
		match node {
			Node::Group(g) => Self::collect_group(g, info),

			Node::Path(_) => {}

			Node::Text(t) => {
				for chunk in t.chunks() {
					for span in chunk.spans() {
						if !info.fonts.contains(span.font()) {
							info.fonts.insert(span.font().clone());
						}
					}
				}
			}

			Node::Image(i) => {
				if let ImageKind::SVG(sub_tree) = &i.kind() {
					Self::collect_group(sub_tree.root(), info);
				}
			}
		}

		node.subroots(|subroot: &Group| {
			Self::collect_group(subroot, info);
		});
	}
}
