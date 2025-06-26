use ratatui::{text::Line, widgets::Paragraph};
use tokio::sync::oneshot::Sender;
use yazi_config::popup::Position;

#[derive(Default)]
pub struct Confirm {
	pub title: Line<'static>,
	pub body:  Paragraph<'static>,
	pub list:  Paragraph<'static>,

	pub position: Position,
	pub offset:   usize,

	pub(super) callback: Option<Sender<bool>>,
	pub visible:         bool,
}
