use std::str::SplitWhitespace;

use strum::FromRepr;

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

	// Drop
	DropEnter(DndDropEnter),
	DropLeave,
	DropReady(DndDropReady),
	DropData(DndDropData),
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
pub struct DndDropData {
	pub idx:     u8,
	pub payload: Vec<u8>,
}

impl DndEvent {
	pub(crate) fn from_state(s: StateOsc72) -> Option<Self> {
		Some(match s.r#type.unwrap_or_default() {
			// Drag
			b'o' => Self::DragOffer(DndDragOffer { x: s.x?.try_into().ok()?, y: s.y?.try_into().ok()? }),
			b'e' if s.x? == 1 => Self::DragAccept(DndDragAccept { idx: s.y?.try_into().ok()? }),
			b'e' if s.x? == 2 => Self::DragChange(DndDragChange { op: DndOp::from_repr(s.op?)? }),
			b'e' if s.x? == 3 => Self::DragLand,
			b'e' if s.x? == 4 => Self::DragEnd(DndDragEnd { canceled: s.y? != 0 }),
			b'e' if s.x? == 5 => Self::DragSend(DndDragSend { idx: s.y?.try_into().ok()? }),

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
			b'r' => Self::DropData(DndDropData { idx: s.x?.try_into().ok()?, payload: s.payload }),

			_ => return None,
		})
	}
}

// --- Operation
#[derive(Clone, Copy, Debug, Eq, FromRepr, PartialEq)]
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
