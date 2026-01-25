use ratatui::{text::Line, widgets::Paragraph};
use yazi_config::popup::Position;
use yazi_shared::CompletionToken;

#[derive(Default)]
pub struct Confirm {
	pub title: Line<'static>,
	pub body:  Paragraph<'static>,
	pub list:  Paragraph<'static>,

	pub position: Position,
	pub offset:   usize,

	pub token:   CompletionToken,
	pub visible: bool,
}
