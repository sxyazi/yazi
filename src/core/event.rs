use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Result;
use crossterm::event::KeyEvent;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use super::{files::{File, FilesOp}, input::InputOpt, manager::PreviewData, select::SelectOpt};
use crate::config::{keymap::{Control, KeymapLayer}, open::Opener};

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
	Open(Vec<(PathBuf, String)>, Option<Opener>),
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
		$crate::core::Event::Key($key).emit();
	};
	(Render) => {
		$crate::core::Event::Render(format!("{}:{}", file!(), line!())).emit();
	};
	(Resize($cols:expr, $rows:expr)) => {
		$crate::core::Event::Resize($cols, $rows).emit();
	};
	(Stop($state:expr)) => {{
		let (tx, rx) = tokio::sync::oneshot::channel();
		$crate::core::Event::Stop($state, Some(tx)).wait(rx)
	}};
	(Ctrl($exec:expr, $layer:expr)) => {
		$crate::core::Event::Ctrl($exec, $layer).emit();
	};

	(Cd($op:expr)) => {
		$crate::core::Event::Cd($op).emit();
	};
	(Files($op:expr)) => {
		$crate::core::Event::Files($op).emit();
	};
	(Pages($page:expr)) => {
		$crate::core::Event::Pages($page).emit();
	};
	(Mimetype($mimes:expr)) => {
		$crate::core::Event::Mimetype($mimes).emit();
	};
	(Hover) => {
		$crate::core::Event::Hover(None).emit();
	};
	(Hover($file:expr)) => {
		$crate::core::Event::Hover(Some($file)).emit();
	};
	(Preview($path:expr, $mime:expr, $data:expr)) => {
		$crate::core::Event::Preview($path, $mime, $data).emit();
	};

	(Select($opt:expr)) => {{
		let (tx, rx) = tokio::sync::oneshot::channel();
		$crate::core::Event::Select($opt, tx).wait(rx)
	}};
	(Input($opt:expr)) => {{
		let (tx, rx) = tokio::sync::oneshot::channel();
		$crate::core::Event::Input($opt, tx).wait(rx)
	}};

	(Open($targets:expr, $opener:expr)) => {
		$crate::core::Event::Open($targets, $opener).emit();
	};
	(Progress($percent:expr, $tasks:expr)) => {
		$crate::core::Event::Progress($percent, $tasks).emit();
	};

	($event:ident) => {
		$crate::core::Event::$event.emit();
	};
}
