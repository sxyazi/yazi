use anyhow::Result;
use tokio::sync::oneshot::Sender;

use super::SelectOpt;
use crate::Position;

#[derive(Default)]
pub struct Select {
	title:            String,
	pub(super) items: Vec<String>,
	pub position:     Position,

	pub(super) offset:   usize,
	pub(super) cursor:   usize,
	pub(super) callback: Option<Sender<Result<usize>>>,

	pub visible: bool,
}

impl Select {
	pub fn show(&mut self, opt: SelectOpt, tx: Sender<Result<usize>>) {
		self.close(false);

		self.title = opt.title;
		self.items = opt.items;
		self.position = opt.position;

		self.callback = Some(tx);
		self.visible = true;
	}

	#[inline]
	pub fn window(&self) -> &[String] {
		let end = (self.offset + self.limit()).min(self.items.len());
		&self.items[self.offset..end]
	}

	#[inline]
	pub fn limit(&self) -> usize { self.items.len().min(5) }
}

impl Select {
	#[inline]
	pub fn title(&self) -> String { self.title.clone() }

	#[inline]
	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }
}
