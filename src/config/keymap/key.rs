use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{de::Visitor, Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key {
	pub code:  KeyCode,
	pub shift: bool,
	pub ctrl:  bool,
	pub alt:   bool,
}

impl Default for Key {
	fn default() -> Self { Self { code: KeyCode::Null, shift: false, ctrl: false, alt: false } }
}

impl From<&str> for Key {
	fn from(value: &str) -> Self {
		let mut key = Default::default();
		if value.is_empty() {
			return key;
		}

		if !value.starts_with("<") || !value.ends_with(">") {
			let c = value.chars().next().unwrap();
			key.code = KeyCode::Char(c);
			key.shift = c.is_ascii_uppercase();
			return key;
		}

		let mut it = value[1..value.len() - 1].split_inclusive('-').peekable();
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
				_ => {}
			}
		}
		key
	}
}

impl From<KeyEvent> for Key {
	fn from(value: KeyEvent) -> Self {
		let shift = if let KeyCode::Char(c) = value.code { c.is_ascii_uppercase() } else { false };

		Self {
			code:  value.code,
			shift: shift || value.modifiers.contains(KeyModifiers::SHIFT),
			ctrl:  value.modifiers.contains(KeyModifiers::CONTROL),
			alt:   value.modifiers.contains(KeyModifiers::ALT),
		}
	}
}

impl<'de> Deserialize<'de> for Key {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct KeyVisitor;

		impl<'de> Visitor<'de> for KeyVisitor {
			type Value = Key;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				formatter.write_str("a key string, e.g. <C-a>")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				Ok(Key::from(value))
			}
		}

		deserializer.deserialize_str(KeyVisitor)
	}
}
