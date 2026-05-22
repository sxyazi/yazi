use crate::{Result, bail, event::Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseEvent {
	pub kind:      MouseEventKind,
	pub column:    u16,
	pub row:       u16,
	pub modifiers: Modifiers,
}

// --- Kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventKind {
	Down(MouseButton),
	Up(MouseButton),
	Drag(MouseButton),
	Moved,
	ScrollDown,
	ScrollUp,
	ScrollLeft,
	ScrollRight,
}

impl MouseEventKind {
	/// Cb is the byte of a mouse input that contains the button being used, the
	/// key modifiers being held and whether the mouse is dragging or not.
	///
	/// Bit layout of cb, from low to high:
	/// - button number (bits 0–1)
	/// - shift (bit 2)
	/// - meta/alt (bit 3)
	/// - control (bit 4)
	/// - mouse is dragging (bit 5)
	/// - button number (bits 6–7)
	pub(crate) fn from_cb(cb: u8) -> Result<(Self, Modifiers)> {
		let button = (cb & 0b0000_0011) | ((cb & 0b1100_0000) >> 4);
		let dragging = cb & 0b0010_0000 == 0b0010_0000;

		let kind = match (button, dragging) {
			(0, false) => Self::Down(MouseButton::Left),
			(1, false) => Self::Down(MouseButton::Middle),
			(2, false) => Self::Down(MouseButton::Right),
			(0, true) => Self::Drag(MouseButton::Left),
			(1, true) => Self::Drag(MouseButton::Middle),
			(2, true) => Self::Drag(MouseButton::Right),
			(3, false) => Self::Up(MouseButton::Left),
			(3, true) | (4, true) | (5, true) => Self::Moved,
			(4, false) => Self::ScrollUp,
			(5, false) => Self::ScrollDown,
			(6, false) => Self::ScrollLeft,
			(7, false) => Self::ScrollRight,
			_ => bail!(),
		};

		let mut modifiers = Modifiers::empty();
		if cb & 0b0000_0100 == 0b0000_0100 {
			modifiers |= Modifiers::SHIFT;
		}
		if cb & 0b0000_1000 == 0b0000_1000 {
			modifiers |= Modifiers::ALT;
		}
		if cb & 0b0001_0000 == 0b0001_0000 {
			modifiers |= Modifiers::CONTROL;
		}

		Ok((kind, modifiers))
	}
}

// --- Button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
	Left,
	Right,
	Middle,
}
