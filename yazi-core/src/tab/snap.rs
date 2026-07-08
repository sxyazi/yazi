use yazi_shared::url::UrlBuf;

use crate::tab::Tab;

pub struct TabSnap {
	pub hovered:  Option<UrlBuf>,
	pub selected: Vec<UrlBuf>,
}

impl From<&Tab> for TabSnap {
	fn from(value: &Tab) -> Self {
		Self {
			hovered:  value.hovered_url().cloned(),
			selected: value.selected.urls().cloned().collect(),
		}
	}
}
