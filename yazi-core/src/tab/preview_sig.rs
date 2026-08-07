use ratatui_core::layout::Rect;
use yazi_config::LAYOUT;
use yazi_fs::{FsHash64, file::{File, FileSig}};

#[derive(Clone, Copy, Debug, Hash)]
pub struct PreviewSig<'a> {
	file: FileSig<'a>,
	mime: &'a str,
	area: Rect,
}

impl<'a> PreviewSig<'a> {
	pub fn new(file: &'a File, mime: &'a str) -> Self {
		Self { file: FileSig(file), mime, area: LAYOUT.get().preview }
	}
}

impl FsHash64 for PreviewSig<'_> {}
