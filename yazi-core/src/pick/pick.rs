use anyhow::Result;
use tokio::sync::oneshot::Sender;
use yazi_config::{YAZI, popup::Position};

#[derive(Default)]
pub struct Pick {
	pub(super) title: String,
	pub(super) items: Vec<String>,
	pub position:     Position,

	pub(super) offset:   usize,
	pub(super) cursor:   usize,
	pub(super) callback: Option<Sender<Result<usize>>>,

	pub visible: bool,
}

impl Pick {
	#[inline]
	pub fn window(&self) -> &[String] {
		let end = (self.offset + self.limit()).min(self.items.len());
		&self.items[self.offset..end]
	}

	#[inline]
	pub(super) fn limit(&self) -> usize {
		self.position.offset.height.saturating_sub(YAZI.pick.border()) as usize
	}
}

impl Pick {
	#[inline]
	pub fn title(&self) -> String { self.title.clone() }

	#[inline]
	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }
}
