use std::collections::BTreeMap;

use crossterm::event::KeyEvent;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use super::Exec;
use crate::{files::FilesOp, fs::{Cha, Url}, term::Term, Layer, RoCell};

static TX: RoCell<UnboundedSender<Event>> = RoCell::new();

pub enum Event {
	Quit(bool), // no-cwd-file
	Key(KeyEvent),
	Paste(String),
	Render(String),
	Resize(u16, u16),
	Call(Vec<Exec>, Layer),

	// Manager
	Files(FilesOp),
	Pages(usize),
	Mimetype(BTreeMap<Url, String>),
	Preview(PreviewLock),
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
	(Key($key:expr)) => {
		$crate::event::Event::Key($key).emit();
	};
	(Render) => {
		$crate::event::Event::Render(format!("{}:{}", file!(), line!())).emit();
	};
	(Resize($cols:expr, $rows:expr)) => {
		$crate::event::Event::Resize($cols, $rows).emit();
	};
	(Call($exec:expr, $layer:expr)) => {
		$crate::event::Event::Call($exec, $layer).emit();
	};

	(Files($op:expr)) => {
		$crate::event::Event::Files($op).emit();
	};
	(Pages($page:expr)) => {
		$crate::event::Event::Pages($page).emit();
	};
	(Mimetype($mimes:expr)) => {
		$crate::event::Event::Mimetype($mimes).emit();
	};
	(Preview($lock:expr)) => {
		$crate::event::Event::Preview($lock).emit();
	};

	(Open($targets:expr, $opener:expr)) => {
		$crate::event::Event::Open($targets, $opener).emit();
	};

	($event:ident) => {
		$crate::event::Event::$event.emit();
	};
}

// TODO: remove this
pub struct PreviewLock {
	pub url:  Url,
	pub cha:  Option<Cha>,
	pub skip: usize,
	pub data: PreviewData,
}

#[derive(Debug)]
pub enum PreviewData {
	Folder,
	Text(String),
	Image,
}

impl PreviewLock {
	#[inline]
	pub fn is_image(&self) -> bool { matches!(self.data, PreviewData::Image) }

	#[inline]
	pub fn is_folder(&self) -> bool { matches!(self.data, PreviewData::Folder) }
}
