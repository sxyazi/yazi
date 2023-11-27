use std::{collections::BTreeMap, ffi::OsString};

use anyhow::Result;
use crossterm::event::KeyEvent;
use tokio::sync::{mpsc::{self, UnboundedSender}, oneshot};
use yazi_config::{open::Opener, popup::InputOpt};
use yazi_shared::{fs::Url, term::Term, Exec, InputError, Layer, RoCell};

use super::files::FilesOp;
use crate::{preview::PreviewLock, tasks::TasksProgress};

static TX: RoCell<UnboundedSender<Event>> = RoCell::new();

pub enum Event {
	Quit(bool), // no-cwd-file
	Key(KeyEvent),
	Paste(String),
	Render(String),
	Resize(u16, u16),
	Stop(bool, Option<oneshot::Sender<()>>),
	Call(Vec<Exec>, Layer),

	// Manager
	Files(FilesOp),
	Pages(usize),
	Mimetype(BTreeMap<Url, String>),
	Preview(PreviewLock),

	// Input
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
		rx.await.unwrap_or_else(|_| Term::goodbye(|| false))
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

	(Files($op:expr)) => {
		$crate::Event::Files($op).emit();
	};
	(Pages($page:expr)) => {
		$crate::Event::Pages($page).emit();
	};
	(Mimetype($mimes:expr)) => {
		$crate::Event::Mimetype($mimes).emit();
	};
	(Preview($lock:expr)) => {
		$crate::Event::Preview($lock).emit();
	};

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
