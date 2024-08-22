use ratatui::widgets::Paragraph;
use tokio::sync::oneshot::Sender;
use yazi_config::popup::Position;

#[derive(Default)]
pub struct Confirm {
	pub title:   String,
	pub content: Paragraph<'static>,
	pub list:    Paragraph<'static>,

	pub offset:   usize,
	pub position: Position,

	pub(super) callback: Option<Sender<bool>>,
	pub visible:         bool,
}
