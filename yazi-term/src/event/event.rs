use crate::{Dimension, event::{DndEvent, KeyEvent, MouseEvent}};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
	Key(KeyEvent),
	Mouse(MouseEvent),
	Resize(Dimension),
	FocusIn,
	FocusOut,
	Paste(String),
	Dnd(DndEvent),
}
