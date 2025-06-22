use anyhow::Result;
use tokio::sync::oneshot::Sender;
use yazi_config::popup::Position;

use crate::Scrollable;

#[derive(Default)]
pub struct Pick {
	pub(super) title: String,
	pub(super) items: Vec<String>,
	pub position:     Position,

	pub(super) offset:   usize,
	pub cursor:          usize,
	pub(super) callback: Option<Sender<Result<usize>>>,

	pub visible: bool,
}

impl Pick {
	#[inline]
	pub fn title(&self) -> &str { &self.title }

	#[inline]
	pub fn window(&self) -> impl Iterator<Item = (usize, &str)> {
		self.items.iter().map(AsRef::as_ref).enumerate().skip(self.offset).take(self.limit())
	}
}
