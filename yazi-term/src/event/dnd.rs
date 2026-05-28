use std::str::SplitWhitespace;

use base64::Engine;
use strum::{FromRepr, IntoStaticStr};
use yazi_shim::base64::BASE64_SANE;

use crate::parser::StateOsc72;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DndEvent {
	// Drag
	DragOffer(DndDragOffer),
	DragAccept(DndDragAccept),
	DragChange(DndDragChange),
	DragLand,
	DragEnd(DndDragEnd),
	DragSend(DndDragSend),
	DragError(DndDragError),

	// Drop
	DropEnter(DndDropEnter),
	DropLeave,
	DropReady(DndDropReady),
	DropArrive(DndDropArrive),
	DropError(DndDropError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DndDragOffer {
	pub x: u32,
	pub y: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DndDragAccept {
	pub idx: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DndDragChange {
	pub op: DndOp,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DndDragEnd {
	pub canceled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DndDragSend {
	pub idx: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DndDragError {
	pub idx:  u8,
	pub name: String,
	pub desc: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DndDropEnter {
	pub x:     u32,
	pub y:     u32,
	pub op:    DndOp,
	pub mimes: DndMimeList,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DndDropReady {
	pub x:     u32,
	pub y:     u32,
	pub op:    DndOp,
	pub mimes: DndMimeList,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DndDropArrive {
	pub idx:  u8,
	pub data: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DndDropError {
	pub idx:  u8,
	pub name: String,
	pub desc: String,
}

impl DndEvent {
	pub fn r#type(&self) -> &'static str {
		match self {
			Self::DragOffer(_) => "offer",
			Self::DragAccept(_) => "accept",
			Self::DragChange(_) => "change",
			Self::DragLand => "land",
			Self::DragEnd(_) => "end",
			Self::DragSend(_) => "send",
			Self::DragError(_) => "error",

			Self::DropEnter(_) => "enter",
			Self::DropLeave => "leave",
			Self::DropReady(_) => "ready",
			Self::DropArrive(_) => "arrive",
			Self::DropError(_) => "error",
		}
	}

	pub fn x(&self) -> Option<u32> {
		match self {
			Self::DragOffer(e) => Some(e.x),
			Self::DropEnter(e) => Some(e.x),
			Self::DropReady(e) => Some(e.x),
			_ => None,
		}
	}

	pub fn y(&self) -> Option<u32> {
		match self {
			Self::DragOffer(e) => Some(e.y),
			Self::DropEnter(e) => Some(e.y),
			Self::DropReady(e) => Some(e.y),
			_ => None,
		}
	}

	pub fn idx(&self) -> Option<u8> {
		match self {
			Self::DragAccept(e) => Some(e.idx),
			Self::DragSend(e) => Some(e.idx),
			Self::DragError(e) => Some(e.idx),
			Self::DropArrive(e) => Some(e.idx),
			Self::DropError(e) => Some(e.idx),
			_ => None,
		}
	}

	pub fn op(&self) -> Option<DndOp> {
		match self {
			Self::DragChange(e) => Some(e.op),
			Self::DropEnter(e) => Some(e.op),
			Self::DropReady(e) => Some(e.op),
			_ => None,
		}
	}

	pub fn mimes(&self) -> Option<&DndMimeList> {
		match self {
			Self::DropEnter(e) => Some(&e.mimes),
			Self::DropReady(e) => Some(&e.mimes),
			_ => None,
		}
	}

	pub fn is_drag(&self) -> bool {
		matches!(
			self,
			Self::DragOffer(_)
				| Self::DragAccept(_)
				| Self::DragChange(_)
				| Self::DragLand
				| Self::DragEnd(_)
				| Self::DragSend(_)
				| Self::DragError(_)
		)
	}

	pub(crate) fn from_state(s: StateOsc72) -> Option<Self> {
		Some(match s.r#type.unwrap_or_default() {
			// Drag
			b'o' => Self::DragOffer(DndDragOffer { x: s.x?.try_into().ok()?, y: s.y?.try_into().ok()? }),
			b'e' if s.x? == 1 => Self::DragAccept(DndDragAccept { idx: s.y?.try_into().ok()? }),
			b'e' if s.x? == 2 => Self::DragChange(DndDragChange { op: DndOp::from_repr(s.op?)? }),
			b'e' if s.x? == 3 => Self::DragLand,
			b'e' if s.x? == 4 => Self::DragEnd(DndDragEnd { canceled: s.y? != 0 }),
			b'e' if s.x? == 5 => Self::DragSend(DndDragSend { idx: s.y?.try_into().ok()? }),
			b'E' => {
				let (name, desc) = parse_error(s.payload)?;
				Self::DragError(DndDragError { idx: s.y?.try_into().ok()?, name, desc })
			}

			// Drop
			b'm' if s.x == Some(-1) && s.y == Some(-1) => Self::DropLeave,
			b'm' => Self::DropEnter(DndDropEnter {
				x:     s.x?.try_into().ok()?,
				y:     s.y?.try_into().ok()?,
				op:    DndOp::from_repr(s.op?)?,
				mimes: DndMimeList::new(s.payload)?,
			}),
			b'M' => Self::DropReady(DndDropReady {
				x:     s.x?.try_into().ok()?,
				y:     s.y?.try_into().ok()?,
				op:    DndOp::from_repr(s.op?)?,
				mimes: DndMimeList::new(s.payload)?,
			}),
			b'r' => Self::DropArrive(DndDropArrive {
				idx:  s.x?.try_into().ok()?,
				data: BASE64_SANE.decode(&s.payload).ok()?,
			}),
			b'R' => {
				let (name, desc) = parse_error(s.payload)?;
				Self::DropError(DndDropError { idx: s.x?.try_into().ok()?, name, desc })
			}

			_ => return None,
		})
	}
}

// --- Operation
#[derive(Clone, Copy, Debug, Eq, FromRepr, IntoStaticStr, PartialEq)]
#[repr(u8)]
pub enum DndOp {
	Copy   = 1,
	Move   = 2,
	Either = 3,
}

// --- MIME list
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DndMimeList(String);

impl DndMimeList {
	pub fn new(b: Vec<u8>) -> Option<Self> { Some(Self(String::from_utf8(b).ok()?)) }

	pub fn iter(&self) -> SplitWhitespace<'_> { self.0.split_whitespace() }
}

// --- Error payload parsing
fn parse_error(payload: Vec<u8>) -> Option<(String, String)> {
	let s = String::from_utf8(payload).ok()?;
	Some(match s.split_once(':') {
		Some((name, desc)) => (name.to_owned(), desc.to_owned()),
		None if s.is_empty() => return None,
		None => (s, String::new()),
	})
}
