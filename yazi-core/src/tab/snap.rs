use yazi_fs::file::File;

use crate::tab::Tab;

pub struct TabSnap {
	pub(crate) hovered:  Option<File>,
	pub(crate) selected: Vec<File>,
}

impl From<&Tab> for TabSnap {
	fn from(value: &Tab) -> Self {
		Self { hovered: value.hovered().cloned(), selected: value.selected.files().cloned().collect() }
	}
}
