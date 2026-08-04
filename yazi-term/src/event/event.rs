use crate::{Dimension, event::{ClipboardEvent, DndEvent, KeyEvent, MouseEvent, Report}};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
	Key(KeyEvent),
	Mouse(MouseEvent),
	Resize(Dimension),
	FocusIn,
	FocusOut,
	Paste(String),
	Dnd(DndEvent),
	Clipboard(ClipboardEvent),
	Report(Report),
}
