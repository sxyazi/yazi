use anyhow::bail;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String")]
pub struct Key {
	pub code:  KeyCode,
	pub shift: bool,
	pub ctrl:  bool,
	pub alt:   bool,
}

impl Key {
	#[inline]
	pub fn plain(&self) -> Option<char> {
		match self.code {
			KeyCode::Char(c) if !self.ctrl && !self.alt => Some(c),
			_ => None,
		}
	}

	#[inline]
	pub fn is_enter(&self) -> bool {
		matches!(self, Key { code: KeyCode::Enter, shift: false, ctrl: false, alt: false })
	}
}

impl Default for Key {
	fn default() -> Self { Self { code: KeyCode::Null, shift: false, ctrl: false, alt: false } }
}

impl From<KeyEvent> for Key {
	fn from(value: KeyEvent) -> Self {
		// For alphabet:
		//   Unix    :  <S-a> => Char("A") + SHIFT
		//   Windows :  <S-a> => Char("A") + SHIFT
		//
		// For non-alphabet:
		//   Unix    :  <S-`> => Char("~") + NULL
		//   Windows :  <S-`> => Char("~") + SHIFT
		//
		// So we detect `Char("~") + SHIFT`, and change it to `Char("~") + NULL`
		// for consistent behavior between OSs.

		let shift = match (value.code, value.modifiers) {
			(KeyCode::Char(c), _) => c.is_ascii_uppercase(),
			(_, m) => m.contains(KeyModifiers::SHIFT),
		};

		Self {
			code: value.code,
			shift,
			ctrl: value.modifiers.contains(KeyModifiers::CONTROL),
			alt: value.modifiers.contains(KeyModifiers::ALT),
		}
	}
}

impl TryFrom<String> for Key {
	type Error = anyhow::Error;

	fn try_from(s: String) -> Result<Self, Self::Error> {
		if s.is_empty() {
			bail!("empty key")
		}

		let mut key = Self::default();
		if !s.starts_with('<') || !s.ends_with('>') {
			let c = s.chars().next().unwrap();
			key.code = KeyCode::Char(c);
			key.shift = c.is_ascii_uppercase();
			return Ok(key);
		}

		let mut it = s[1..s.len() - 1].split_inclusive('-').peekable();
		while let Some(x) = it.next() {
			match x {
				"S-" => key.shift = true,
				"C-" => key.ctrl = true,
				"A-" => key.alt = true,

				"Space" => key.code = KeyCode::Char(' '),
				"Backspace" => key.code = KeyCode::Backspace,
				"Enter" => key.code = KeyCode::Enter,
				"Left" => key.code = KeyCode::Left,
				"Right" => key.code = KeyCode::Right,
				"Up" => key.code = KeyCode::Up,
				"Down" => key.code = KeyCode::Down,
				"Home" => key.code = KeyCode::Home,
				"End" => key.code = KeyCode::End,
				"PageUp" => key.code = KeyCode::PageUp,
				"PageDown" => key.code = KeyCode::PageDown,
				"Tab" => key.code = KeyCode::Tab,
				"Delete" => key.code = KeyCode::Delete,
				"Insert" => key.code = KeyCode::Insert,
				"F1" => key.code = KeyCode::F(1),
				"F2" => key.code = KeyCode::F(2),
				"F3" => key.code = KeyCode::F(3),
				"F4" => key.code = KeyCode::F(4),
				"F5" => key.code = KeyCode::F(5),
				"F6" => key.code = KeyCode::F(6),
				"F7" => key.code = KeyCode::F(7),
				"F8" => key.code = KeyCode::F(8),
				"F9" => key.code = KeyCode::F(9),
				"F10" => key.code = KeyCode::F(10),
				"F11" => key.code = KeyCode::F(11),
				"F12" => key.code = KeyCode::F(12),
				"Esc" => key.code = KeyCode::Esc,

				c if it.peek().is_none() => {
					key.code = KeyCode::Char(c.chars().next().unwrap());
				}
				k => bail!("unknown key: {k}"),
			}
		}

		if key.code == KeyCode::Null {
			bail!("empty key")
		}
		Ok(key)
	}
}

impl ToString for Key {
	fn to_string(&self) -> String {
		if let Some(c) = self.plain() {
			let c = if self.shift { c.to_ascii_uppercase() } else { c };
			return if c == ' ' { "<Space>".to_string() } else { c.to_string() };
		}

		let mut s = "<".to_string();
		if self.ctrl {
			s += "C-";
		}
		if self.alt {
			s += "A-";
		}
		if self.shift && !matches!(self.code, KeyCode::Char(_)) {
			s += "S-";
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
			KeyCode::F(1) => "F1",
			KeyCode::F(2) => "F2",
			KeyCode::F(3) => "F3",
			KeyCode::F(4) => "F4",
			KeyCode::F(5) => "F5",
			KeyCode::F(6) => "F6",
			KeyCode::F(7) => "F7",
			KeyCode::F(8) => "F8",
			KeyCode::F(9) => "F9",
			KeyCode::F(10) => "F10",
			KeyCode::F(11) => "F11",
			KeyCode::F(12) => "F12",
			KeyCode::Esc => "Esc",

			KeyCode::Char(' ') => "Space",
			KeyCode::Char(c) => {
				s.push(if self.shift { c.to_ascii_uppercase() } else { c });
				""
			}
			_ => "Unknown",
		};

		s + code + ">"
	}
}
