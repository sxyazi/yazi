use std::str::SplitWhitespace;

use strum::{FromRepr, IntoStaticStr};

use crate::parser::{Osc5522Status, StateOsc5522};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClipboardEvent {
	ReadMimetypes(ClipboardPaste),
	ReadData(ClipboardRead),
	ReadError(ClipboardError),
	WriteSuccess,
	WriteError(ClipboardError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardPaste {
	pub mimes:   ClipboardMimeList,
	pub primary: bool,
	pub pw:      Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardData {
	pub mime: Vec<u8>,
	pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardRead {
	pub mimes:   ClipboardMimeList,
	pub primary: bool,
	pub data:    Vec<ClipboardData>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardError {
	pub name: String,
}

impl ClipboardEvent {
	pub fn r#type(&self) -> &'static str {
		match self {
			Self::ReadMimetypes(_) => "read_mimetypes",
			Self::ReadData(_) => "read_data",
			Self::ReadError(_) => "read_error",
			Self::WriteSuccess => "write_success",
			Self::WriteError(_) => "write_error",
		}
	}

	pub fn mimes(&self) -> Option<&ClipboardMimeList> {
		match self {
			Self::ReadMimetypes(e) => Some(&e.mimes),
			Self::ReadData(e) => Some(&e.mimes),
			_ => None,
		}
	}

	pub fn primary(&self) -> Option<bool> {
		match self {
			Self::ReadMimetypes(e) => Some(e.primary),
			_ => None,
		}
	}

	pub fn pw(&self) -> Option<String> {
		match self {
			Self::ReadMimetypes(e) => Some(String::from_utf8_lossy(&e.pw).into_owned()),
			_ => None,
		}
	}

	pub fn text(&self) -> Option<String> {
		match self {
			Self::ReadData(e) if let Some(t) = e.data.iter().find(|e| e.mime == b"text/plain") => {
				Some(String::from_utf8_lossy(&t.data).into_owned())
			}
			_ => None,
		}
	}

	pub fn is_paste_offer(&self) -> bool {
		match self {
			Self::ReadMimetypes(_) => true,
			_ => false,
		}
	}

	pub fn is_read(&self) -> bool {
		match self {
			Self::ReadError(_) | Self::ReadData(_) => true,
			_ => false,
		}
	}

	pub(crate) fn from_state(s: StateOsc5522) -> Option<Self> {
		Some(match s {
			StateOsc5522 { read: true, status: Some(Osc5522Status::DONE), idx: 0, mime, .. }
				if mime.first()? == b"." =>
			{
				ClipboardEvent::ReadMimetypes(ClipboardPaste {
					mimes:   ClipboardMimeList::new(s.payload.first()?.to_owned())?,
					primary: s.primary,
					pw:      s.pw,
				})
			}
			StateOsc5522 { read: true, status: Some(Osc5522Status::DONE), .. } => {
				let mut mimes = Vec::new();
				let mut data = Vec::new();
				for (mime, payload) in s.mime.iter().zip(s.payload.iter()) {
					data.push(ClipboardData { mime: mime.to_owned(), data: payload.to_owned() });
					mimes.extend(mime);
					mimes.push(b' ');
				}
				ClipboardEvent::ReadData(ClipboardRead {
					mimes: ClipboardMimeList::new(mimes)?,
					primary: s.primary,
					data,
				})
			}
			StateOsc5522 { read: true, .. } => {
				Self::ReadError(ClipboardError { name: parse_error(s.status)? })
			}
			StateOsc5522 { read: false, status: Some(Osc5522Status::DONE), .. } => {
				ClipboardEvent::WriteSuccess
			}
			StateOsc5522 { read: false, .. } => {
				Self::WriteError(ClipboardError { name: parse_error(s.status)? })
			}
		})
	}
}

// --- Operation
#[derive(Clone, Copy, Debug, Eq, FromRepr, IntoStaticStr, PartialEq)]
#[repr(u8)]
pub enum ClipboardType {
	Read  = 1,
	Write = 2,
}

// --- MIME list
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardMimeList(String);

impl ClipboardMimeList {
	pub fn new(b: Vec<u8>) -> Option<Self> { Some(Self(String::from_utf8(b).ok()?)) }

	pub fn iter(&self) -> SplitWhitespace<'_> { self.0.split_whitespace() }
}

// --- Error payload parsing
fn parse_error(status: Option<Osc5522Status>) -> Option<String> {
	match status {
		Some(Osc5522Status::ENOSYS) => Some("ENOSYS".to_string()),
		Some(Osc5522Status::EPERM) => Some("EPERM".to_string()),
		Some(Osc5522Status::EBUSY) => Some("EBUSY".to_string()),
		Some(Osc5522Status::EIO) => Some("EIO".to_string()),
		Some(Osc5522Status::EINVAL) => Some("EINVAL".to_string()),
		_ => None,
	}
}
