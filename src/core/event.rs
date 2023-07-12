use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::KeyEvent;
use tokio::sync::{mpsc::Sender, oneshot};

use super::{files::FilesOp, input::InputOpt, manager::PreviewData};

static mut TX: Option<Sender<Event>> = None;

pub enum Event {
	Quit,
	Stop(bool, Option<oneshot::Sender<()>>),
	Key(KeyEvent),
	Render(String),
	Resize(u16, u16),

	Cd(PathBuf),
	Refresh,
	Files(FilesOp),
	Hover,
	Mimetype(PathBuf, String),
	Preview(PathBuf, PreviewData),

	Input(InputOpt, oneshot::Sender<Result<String>>),

	Open(Vec<PathBuf>),
	Progress(u8, u32),
}

impl Event {
	#[inline]
	pub fn init(tx: Sender<Event>) {
		unsafe {
			TX.replace(tx);
		}
	}

	#[inline]
	pub fn emit(self) {
		let tx = unsafe { TX.as_ref().unwrap() };
		tokio::spawn(async {
			tx.send(self).await.ok();
		});
	}

	pub async fn wait<T>(self, rx: oneshot::Receiver<T>) -> T {
		let tx = unsafe { TX.as_ref().unwrap() };
		tx.send(self).await.ok();
		rx.await.unwrap()
	}
}

#[macro_export]
macro_rules! emit {
	(Stop($state:expr)) => {{
		let (tx, rx) = tokio::sync::oneshot::channel();
		$crate::core::Event::Stop($state, Some(tx)).wait(rx)
	}};
	(Key($key:expr)) => {
		$crate::core::Event::Key($key).emit();
	};
	(Render) => {
		$crate::core::Event::Render(format!("{}:{}", file!(), line!())).emit();
	};
	(Resize($cols:expr, $rows:expr)) => {
		$crate::core::Event::Resize($cols, $rows).emit();
	};

	(Cd($op:expr)) => {
		$crate::core::Event::Cd($op).emit();
	};
	(Files($op:expr)) => {
		$crate::core::Event::Files($op).emit();
	};
	(Mimetype($path:expr, $mime:expr)) => {
		$crate::core::Event::Mimetype($path, $mime).emit();
	};
	(Preview($path:expr, $data:expr)) => {
		$crate::core::Event::Preview($path, $data).emit();
	};

	(Input($opt:expr)) => {{
		let (tx, rx) = tokio::sync::oneshot::channel();
		$crate::core::Event::Input($opt, tx).wait(rx)
	}};

	(Open($files:expr)) => {
		$crate::core::Event::Open($files).emit();
	};
	(Progress($percent:expr, $tasks:expr)) => {
		$crate::core::Event::Progress($percent, $tasks).emit();
	};

	($event:ident) => {
		$crate::core::Event::$event.emit();
	};
}
