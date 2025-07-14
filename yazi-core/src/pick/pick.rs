use anyhow::Result;
use tokio::sync::oneshot::Sender;
use yazi_config::{YAZI, popup::Position};
use yazi_widgets::Scrollable;

#[derive(Default)]
pub struct Pick {
	pub title:    String,
	pub items:    Vec<String>,
	pub position: Position,

	pub offset:   usize,
	pub cursor:   usize,
	pub callback: Option<Sender<Result<usize>>>,

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

impl Scrollable for Pick {
	#[inline]
	fn total(&self) -> usize { self.items.len() }

	#[inline]
	fn limit(&self) -> usize {
		self.position.offset.height.saturating_sub(YAZI.pick.border()) as usize
	}

	#[inline]
	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	#[inline]
	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
