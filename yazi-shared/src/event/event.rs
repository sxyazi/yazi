use std::ffi::OsString;

use crossterm::event::{KeyEvent, MouseEvent};
use tokio::sync::mpsc;

use super::CmdCow;
use crate::RoCell;

static TX: RoCell<mpsc::UnboundedSender<Event>> = RoCell::new();
static RX: RoCell<mpsc::UnboundedReceiver<Event>> = RoCell::new();

#[derive(Debug)]
pub enum Event {
	Call(CmdCow),
	Seq(Vec<CmdCow>),
	Render,
	Key(KeyEvent),
	Mouse(MouseEvent),
	Resize,
	Paste(String),
	Quit(EventQuit),
}

#[derive(Debug, Default)]
pub struct EventQuit {
	pub code:        i32,
	pub no_cwd_file: bool,
	pub selected:    Option<OsString>,
}

impl Event {
	#[inline]
	pub fn init() {
		let (tx, rx) = mpsc::unbounded_channel();
		TX.init(tx);
		RX.init(rx);
	}

	#[inline]
	pub fn take() -> mpsc::UnboundedReceiver<Event> { RX.drop() }

	#[inline]
	pub fn emit(self) { TX.send(self).ok(); }
}
