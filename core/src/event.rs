use std::{collections::BTreeMap, ffi::OsString, path::PathBuf};

use anyhow::Result;
use config::{keymap::{Control, KeymapLayer}, open::Opener};
use crossterm::event::KeyEvent;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use super::{files::{File, FilesOp}, input::InputOpt, manager::PreviewData, select::SelectOpt};

static mut TX: Option<UnboundedSender<Event>> = None;

pub enum Event {
	Quit,
	Key(KeyEvent),
	Paste(String),
	Render(String),
	Resize(u16, u16),
	Stop(bool, Option<oneshot::Sender<()>>),
	Ctrl(Control, KeymapLayer),

	// Manager
	Cd(PathBuf),
	Refresh,
	Files(FilesOp),
	Pages(usize),
	Mimetype(BTreeMap<PathBuf, String>),
	Hover(Option<File>),
	Preview(PathBuf, String, PreviewData),

	// Input
	Select(SelectOpt, oneshot::Sender<Result<usize>>),
	Input(InputOpt, oneshot::Sender<Result<String>>),

	// Tasks
	Open(Vec<(OsString, String)>, Option<Opener>),
	Progress(u8, u32),
}

impl Event {
	#[inline]
	pub fn init(tx: UnboundedSender<Event>) {
		unsafe {
			TX.replace(tx);
		}
	}

	#[inline]
	pub fn emit(self) {
		let tx = unsafe { TX.as_ref().unwrap() };
		tx.send(self).ok();
	}

	pub async fn wait<T>(self, rx: oneshot::Receiver<T>) -> T {
		let tx = unsafe { TX.as_ref().unwrap() };
		tx.send(self).ok();
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

	(Cd($op:expr)) => {
		$crate::Event::Cd($op).emit();
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
	(Preview($path:expr, $mime:expr, $data:expr)) => {
		$crate::Event::Preview($path, $mime, $data).emit();
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
