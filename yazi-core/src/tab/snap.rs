use yazi_fs::file::File;

use crate::tab::Tab;

pub struct TabSnap {
	pub hovered:  Option<File>,
	pub selected: Vec<File>,
}

impl From<&Tab> for TabSnap {
	fn from(value: &Tab) -> Self {
		Self { hovered: value.hovered().cloned(), selected: value.selected.files().cloned().collect() }
	}
}
