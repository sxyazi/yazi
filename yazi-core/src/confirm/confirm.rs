use anyhow::Result;
use tokio::sync::oneshot::Sender;
use yazi_config::popup::Position;

#[derive(Default)]
pub struct Confirm {
	pub(super) title:  String,
	message:           String,
	message_num_lines: usize,

	pub position: Position,

	pub vertical_scroll: usize,
	pub(super) callback: Option<Sender<Result<bool>>>,

	pub visible: bool,
}

impl Confirm {
	#[inline]
	pub fn set_message(&mut self, message: &str) {
		self.message = message.to_string();
		self.message_num_lines = self.message.split('\n').count();
	}

	#[inline]
	pub fn message(&self) -> String { self.message.clone() }

	#[inline]
	pub fn message_num_lines(&self) -> usize { self.message_num_lines }

	#[inline]
	pub fn title(&self) -> String { self.title.clone() }
}
