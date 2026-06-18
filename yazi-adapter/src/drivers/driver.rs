use std::path::PathBuf;

use anyhow::Result;
use ratatui::layout::Rect;
use strum::{Display, IntoStaticStr};

use crate::drivers::{Chafa, Iip, Kgp, KgpOld, Sixel, Ueberzug};

#[derive(Clone, Copy, Debug, Display, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum Driver {
	Kgp,
	KgpOld,
	Iip,
	Sixel,

	// Supported by Überzug++
	X11,
	Wayland,
	Chafa,
}

impl Driver {
	pub async fn image_show<P>(self, path: P, max: Rect) -> Result<Rect>
	where
		P: Into<PathBuf>,
	{
		if max.is_empty() {
			return Ok(Rect::default());
		}

		let path = path.into();
		match self {
			Self::Kgp => Kgp::image_show(path, max).await,
			Self::KgpOld => KgpOld::image_show(path, max).await,
			Self::Iip => Iip::image_show(path, max).await,
			Self::Sixel => Sixel::image_show(path, max).await,
			Self::X11 | Self::Wayland => Ueberzug::image_show(path, max).await,
			Self::Chafa => Chafa::image_show(path, max).await,
		}
	}

	pub fn image_erase(self, area: Rect) -> Result<()> {
		match self {
			Self::Kgp => Kgp::image_erase(area),
			Self::KgpOld => KgpOld::image_erase(area),
			Self::Iip => Iip::image_erase(area),
			Self::Sixel => Sixel::image_erase(area),
			Self::X11 | Self::Wayland => Ueberzug::image_erase(area),
			Self::Chafa => Chafa::image_erase(area),
		}
	}

	pub(crate) fn start(self) { Ueberzug::start(self); }

	pub(crate) fn needs_ueberzug(self) -> bool {
		!matches!(self, Self::Kgp | Self::KgpOld | Self::Iip | Self::Sixel)
	}
}
