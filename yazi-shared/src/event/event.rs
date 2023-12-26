use crossterm::event::KeyEvent;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use super::Exec;
use crate::{term::Term, Layer, RoCell};

static TX: RoCell<UnboundedSender<Event>> = RoCell::new();

pub enum Event {
	Quit(bool), // no-cwd-file
	Key(KeyEvent),
	Paste(String),
	Render(String),
	Resize(u16, u16),
	Call(Vec<Exec>, Layer),

	// Manager
	Pages(usize),
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
	(Quit($no_cwd_file:expr)) => {
		$crate::event::Event::Quit($no_cwd_file).emit();
	};
	(Render) => {
		$crate::event::Event::Render(format!("{}:{}", file!(), line!())).emit();
	};
	(Call($exec:expr, $layer:expr)) => {
		$crate::event::Event::Call($exec, $layer).emit();
	};

	(Pages($page:expr)) => {
		$crate::event::Event::Pages($page).emit();
	};

	($event:ident) => {
		$crate::event::Event::$event.emit();
	};
}
