use crate::{event::{KeyCode, KeyEvent, Modifiers}, parser::Parser};

impl Parser {
	/// Maps a ground-state byte to a [`KeyEvent`], returning [`None`] for ESC
	/// (`\x1B`), multi-byte UTF-8 lead bytes, and continuation / invalid bytes.
	pub(super) fn parse_ground_key(b: u8) -> Option<KeyEvent> {
		match b {
			b'\r' => Some(KeyCode::Enter.into()),
			b'\t' => Some(KeyCode::Tab.into()),
			b'\x7F' => Some(KeyCode::Backspace.into()),
			b'\0' => Some(KeyEvent::new(KeyCode::Char(' '), Modifiers::CONTROL)),
			c @ b'\x01'..=b'\x1A' => {
				Some(KeyEvent::new(KeyCode::Char((c - 0x1 + b'a') as char), Modifiers::CONTROL))
			}
			c @ b'\x1C'..=b'\x1F' => {
				Some(KeyEvent::new(KeyCode::Char((c - 0x1c + b'4') as char), Modifiers::CONTROL))
			}
			b'\x20'..=b'\x7E' => {
				Some(KeyEvent::new(KeyCode::Char(b as char), Modifiers::for_char(b as char)))
			}
			_ => None,
		}
	}
}
