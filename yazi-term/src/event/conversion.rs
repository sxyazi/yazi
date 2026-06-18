use std::{fmt::{Display, Write}, str::FromStr};

use anyhow::bail;

use crate::event::{KeyCode, KeyEvent, Modifiers};

impl FromStr for KeyEvent {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.is_empty() {
			bail!("empty key")
		}

		if !s.starts_with('<') || !s.ends_with('>') {
			let c = s.chars().next().unwrap();
			return Ok(Self::new(KeyCode::Char(c), Modifiers::for_char(c)));
		}

		let mut key: Self = KeyCode::Null.into();
		let mut it = s[1..s.len() - 1].split_inclusive('-').peekable();
		while let Some(next) = it.next() {
			match next.to_ascii_lowercase().as_str() {
				"s-" => key.modifiers |= Modifiers::SHIFT,
				"c-" => key.modifiers |= Modifiers::CONTROL,
				"a-" => key.modifiers |= Modifiers::ALT,
				"d-" => key.modifiers |= Modifiers::SUPER,

				"space" => key.code = KeyCode::Char(' '),
				"backspace" => key.code = KeyCode::Backspace,
				"enter" => key.code = KeyCode::Enter,
				"left" => key.code = KeyCode::Left,
				"right" => key.code = KeyCode::Right,
				"up" => key.code = KeyCode::Up,
				"down" => key.code = KeyCode::Down,
				"home" => key.code = KeyCode::Home,
				"end" => key.code = KeyCode::End,
				"pageup" => key.code = KeyCode::PageUp,
				"pagedown" => key.code = KeyCode::PageDown,
				"tab" => key.code = KeyCode::Tab,
				"delete" => key.code = KeyCode::Delete,
				"insert" => key.code = KeyCode::Insert,
				"f1" => key.code = KeyCode::Fn(1),
				"f2" => key.code = KeyCode::Fn(2),
				"f3" => key.code = KeyCode::Fn(3),
				"f4" => key.code = KeyCode::Fn(4),
				"f5" => key.code = KeyCode::Fn(5),
				"f6" => key.code = KeyCode::Fn(6),
				"f7" => key.code = KeyCode::Fn(7),
				"f8" => key.code = KeyCode::Fn(8),
				"f9" => key.code = KeyCode::Fn(9),
				"f10" => key.code = KeyCode::Fn(10),
				"f11" => key.code = KeyCode::Fn(11),
				"f12" => key.code = KeyCode::Fn(12),
				"f13" => key.code = KeyCode::Fn(13),
				"f14" => key.code = KeyCode::Fn(14),
				"f15" => key.code = KeyCode::Fn(15),
				"f16" => key.code = KeyCode::Fn(16),
				"f17" => key.code = KeyCode::Fn(17),
				"f18" => key.code = KeyCode::Fn(18),
				"f19" => key.code = KeyCode::Fn(19),
				"esc" => key.code = KeyCode::Escape,

				_ => match next {
					s if it.peek().is_none() => {
						let c = s.chars().next().unwrap();
						key.modifiers |= Modifiers::for_char(c);
						key.code = KeyCode::Char(if key.modifiers.contains(Modifiers::SHIFT) {
							c.to_ascii_uppercase()
						} else {
							c
						});
					}
					s => bail!("unknown key: {s}"),
				},
			}
		}

		if key.code == KeyCode::Null {
			bail!("empty key")
		}
		Ok(key)
	}
}

impl Display for KeyEvent {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(c) = self.plain() {
			return if c == ' ' { write!(f, "<Space>") } else { f.write_char(c) };
		}

		write!(f, "<")?;
		if self.modifiers.contains(Modifiers::SUPER) {
			write!(f, "D-")?;
		}
		if self.modifiers.contains(Modifiers::CONTROL) {
			write!(f, "C-")?;
		}
		if self.modifiers.contains(Modifiers::ALT) {
			write!(f, "A-")?;
		}
		if self.modifiers.contains(Modifiers::SHIFT) && !matches!(self.code, KeyCode::Char(_)) {
			write!(f, "S-")?;
		}

		let code = match self.code {
			KeyCode::Backspace => "Backspace",
			KeyCode::Enter => "Enter",
			KeyCode::Left => "Left",
			KeyCode::Right => "Right",
			KeyCode::Up => "Up",
			KeyCode::Down => "Down",
			KeyCode::Home => "Home",
			KeyCode::End => "End",
			KeyCode::PageUp => "PageUp",
			KeyCode::PageDown => "PageDown",
			KeyCode::Tab => "Tab",
			KeyCode::Delete => "Delete",
			KeyCode::Insert => "Insert",
			KeyCode::Fn(1) => "F1",
			KeyCode::Fn(2) => "F2",
			KeyCode::Fn(3) => "F3",
			KeyCode::Fn(4) => "F4",
			KeyCode::Fn(5) => "F5",
			KeyCode::Fn(6) => "F6",
			KeyCode::Fn(7) => "F7",
			KeyCode::Fn(8) => "F8",
			KeyCode::Fn(9) => "F9",
			KeyCode::Fn(10) => "F10",
			KeyCode::Fn(11) => "F11",
			KeyCode::Fn(12) => "F12",
			KeyCode::Fn(13) => "F13",
			KeyCode::Fn(14) => "F14",
			KeyCode::Fn(15) => "F15",
			KeyCode::Fn(16) => "F16",
			KeyCode::Fn(17) => "F17",
			KeyCode::Fn(18) => "F18",
			KeyCode::Fn(19) => "F19",
			KeyCode::Escape => "Esc",

			KeyCode::Char(' ') => "Space",
			KeyCode::Char(c) => {
				f.write_char(c)?;
				""
			}
			_ => "Unknown",
		};

		write!(f, "{code}>")
	}
}
