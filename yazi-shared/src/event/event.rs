use std::ffi::OsString;

use crossterm::event::KeyEvent;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use super::Exec;
use crate::{term::Term, Layer, RoCell};

static TX: RoCell<UnboundedSender<Event>> = RoCell::new();

#[derive(Debug)]
pub enum Event {
	Call(Vec<Exec>, Layer),
	Render,
	Key(KeyEvent),
	Resize(u16, u16),
	Paste(String),
	Quit(Vec<QuitAction>),
}

#[derive(Debug, PartialEq)]
pub enum QuitAction {
	CwdToFile,
	SelectToFile(OsString),
}

impl Event {
	#[inline]
	pub fn init(tx: UnboundedSender<Event>) { TX.init(tx); }

	#[inline]
	pub fn emit(self) { TX.send(self).ok(); }

	pub async fn wait<T>(self, rx: oneshot::Receiver<T>) -> T {
		TX.send(self).ok();
		rx.await.unwrap_or_else(|_| Term::goodbye(|| false))
	}
}

#[macro_export]
macro_rules! emit {
	(Quit($actions:expr)) => {
		$crate::event::Event::Quit($actions).emit();
	};
	(Call($exec:expr, $layer:expr)) => {
		$crate::event::Event::Call($exec, $layer).emit();
	};
	($event:ident) => {
		$crate::event::Event::$event.emit();
	};
}
