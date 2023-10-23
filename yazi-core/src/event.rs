use std::{collections::BTreeMap, ffi::OsString};

use anyhow::Result;
use crossterm::event::KeyEvent;
use tokio::sync::{mpsc::{self, UnboundedSender}, oneshot};
use yazi_config::{keymap::{Exec, KeymapLayer}, open::Opener};
use yazi_shared::{InputError, RoCell, Url};

use super::{files::FilesOp, input::InputOpt, select::SelectOpt};
use crate::{preview::PreviewLock, tasks::TasksProgress};

static TX: RoCell<UnboundedSender<Event>> = RoCell::new();

pub enum Event {
	Quit(bool), // no-cwd-file
	Key(KeyEvent),
	Paste(String),
	Render(String),
	Resize(u16, u16),
	Stop(bool, Option<oneshot::Sender<()>>),
	Call(Vec<Exec>, KeymapLayer),

	// Manager
	Cd(Url),
	Refresh,
	Files(FilesOp),
	Pages(usize),
	Mimetype(BTreeMap<Url, String>),
	Hover(Option<Url>),
	Peek(Option<(usize, Url)>),
	Preview(PreviewLock),

	// Input
	Select(SelectOpt, oneshot::Sender<Result<usize>>),
	Input(InputOpt, mpsc::UnboundedSender<Result<String, InputError>>),

	// Tasks
	Open(Vec<(OsString, String)>, Option<Opener>),
	Progress(TasksProgress),
}

impl Event {
	#[inline]
	pub fn init(tx: UnboundedSender<Event>) { TX.init(tx); }

	#[inline]
	pub fn emit(self) { TX.send(self).ok(); }

	pub async fn wait<T>(self, rx: oneshot::Receiver<T>) -> T {
		TX.send(self).ok();
		rx.await.unwrap_or_else(|_| std::process::exit(0))
	}
}

#[macro_export]
macro_rules! emit {
	(Quit($no_cwd_file:expr)) => {
		$crate::Event::Quit($no_cwd_file).emit();
	};
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
	(Call($exec:expr, $layer:expr)) => {
		$crate::Event::Call($exec, $layer).emit();
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
		$crate::Event::Peek(None).emit();
	};
	(Peek($skip:expr, $url:expr)) => {
		$crate::Event::Peek(Some(($skip, $url))).emit();
	};
	(Preview($lock:expr)) => {
		$crate::Event::Preview($lock).emit();
	};

	(Select($opt:expr)) => {{
		let (tx, rx) = tokio::sync::oneshot::channel();
		$crate::Event::Select($opt, tx).wait(rx)
	}};
	(Input($opt:expr)) => {{
		let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
		$crate::Event::Input($opt, tx).emit();
		rx
	}};

	(Open($targets:expr, $opener:expr)) => {
		$crate::Event::Open($targets, $opener).emit();
	};
	(Progress($progress:expr)) => {
		$crate::Event::Progress($progress).emit();
	};

	($event:ident) => {
		$crate::Event::$event.emit();
	};
}
