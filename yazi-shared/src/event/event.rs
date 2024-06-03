use std::{collections::VecDeque, ffi::OsString};

use crossterm::event::{KeyEvent, MouseEvent};
use tokio::sync::mpsc;

use super::Cmd;
use crate::{Layer, RoCell};

static TX: RoCell<mpsc::UnboundedSender<Event>> = RoCell::new();
static RX: RoCell<mpsc::UnboundedReceiver<Event>> = RoCell::new();

#[derive(Debug)]
pub enum Event {
	Call(Cmd, Layer),
	Seq(VecDeque<Cmd>, Layer),
	Render,
	Key(KeyEvent),
	Mouse(MouseEvent),
	Resize,
	Paste(String),
	Quit(EventQuit),
}

#[derive(Debug, Default)]
pub struct EventQuit {
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

#[macro_export]
macro_rules! emit {
	(Quit($opt:expr)) => {
		$crate::event::Event::Quit($opt).emit();
	};
	(Call($cmd:expr, $layer:expr)) => {
		$crate::event::Event::Call($cmd, $layer).emit();
	};
	(Seq($cmds:expr, $layer:expr)) => {
		$crate::event::Event::Seq($cmds, $layer).emit();
	};
	($event:ident) => {
		$crate::event::Event::$event.emit();
	};
}
