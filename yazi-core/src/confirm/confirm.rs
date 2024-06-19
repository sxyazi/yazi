use anyhow::Result;
use tokio::sync::oneshot::Sender;
use yazi_config::popup::Position;

#[derive(Default)]
pub struct Confirm {
	pub title:   String,
	pub content: String,
	pub lines:   usize,

	pub offset:   usize,
	pub position: Position,

	pub(super) callback: Option<Sender<Result<bool>>>,
	pub visible:         bool,
}
