use std::{collections::BTreeMap, ffi::OsString};

use anyhow::Result;
use config::{keymap::{Control, KeymapLayer}, open::Opener};
use crossterm::event::KeyEvent;
use shared::{RoCell, Url};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use super::{files::{File, FilesOp}, input::InputOpt, select::SelectOpt};
use crate::manager::PreviewLock;

static TX: RoCell<UnboundedSender<Event>> = RoCell::new();

pub enum Event {
	Quit,
	Key(KeyEvent),
	Paste(String),
	Render(String),
	Resize(u16, u16),
	Stop(bool, Option<oneshot::Sender<()>>),
	Ctrl(Control, KeymapLayer),

	// Manager
	Cd(Url),
	Refresh,
	Files(FilesOp),
	Pages(usize),
	Mimetype(BTreeMap<Url, String>),
	Hover(Option<File>),
	Peek(usize, Option<Url>),
	Preview(PreviewLock),

	// Input
	Select(SelectOpt, oneshot::Sender<Result<usize>>),
	Input(InputOpt, oneshot::Sender<Result<String>>),

	// Tasks
	Open(Vec<(OsString, String)>, Option<Opener>),
	Progress(u8, u32),
}

impl Event {
	#[inline]
	pub fn init(tx: UnboundedSender<Event>) { TX.init(tx); }

	#[inline]
	pub fn emit(self) { TX.send(self).ok(); }

	pub async fn wait<T>(self, rx: oneshot::Receiver<T>) -> T {
		TX.send(self).ok();
		rx.await.unwrap()
	}
}

#[macro_export]
macro_rules! emit {
	(Key($key:expr)) => {
		$crate::Event::Key($key).emit();
	};
	(Render) => {
		$crate::Event::Render(format!("{}:{}", file!(), line!())).emit();
	};
	(Resize($cols:expr, $rows:expr)) => {
		$crate::Event::Resize($cols, $rows).emit();
	};
	(Stop($state:expr)) => {{
		let (tx, rx) = tokio::sync::oneshot::channel();
		$crate::Event::Stop($state, Some(tx)).wait(rx)
	}};
	(Ctrl($exec:expr, $layer:expr)) => {
		$crate::Event::Ctrl($exec, $layer).emit();
	};

	(Cd($url:expr)) => {
		$crate::Event::Cd($url).emit();
	};
	(Files($op:expr)) => {
		$crate::Event::Files($op).emit();
	};
	(Pages($page:expr)) => {
		$crate::Event::Pages($page).emit();
	};
	(Mimetype($mimes:expr)) => {
		$crate::Event::Mimetype($mimes).emit();
	};
	(Hover) => {
		$crate::Event::Hover(None).emit();
	};
	(Hover($file:expr)) => {
		$crate::Event::Hover(Some($file)).emit();
	};
	(Peek) => {
		$crate::Event::Peek(0, None).emit();
	};
	(Peek($skip:expr, $url:expr)) => {
		$crate::Event::Peek($skip, Some($url)).emit();
	};
	(Preview($lock:expr)) => {
		$crate::Event::Preview($lock).emit();
	};

	(Select($opt:expr)) => {{
		let (tx, rx) = tokio::sync::oneshot::channel();
		$crate::Event::Select($opt, tx).wait(rx)
	}};
	(Input($opt:expr)) => {{
		let (tx, rx) = tokio::sync::oneshot::channel();
		$crate::Event::Input($opt, tx).wait(rx)
	}};

	(Open($targets:expr, $opener:expr)) => {
		$crate::Event::Open($targets, $opener).emit();
	};
	(Progress($percent:expr, $tasks:expr)) => {
		$crate::Event::Progress($percent, $tasks).emit();
	};

	($event:ident) => {
		$crate::Event::$event.emit();
	};
}
