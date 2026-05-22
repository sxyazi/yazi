use bitflags::bitflags;

use crate::{ParseError, Result, bail, event::Modifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
	pub code:      KeyCode,
	pub kind:      KeyEventKind,
	pub modifiers: Modifiers,
	pub state:     KeyEventState,
}

impl KeyEvent {
	pub const fn new(code: KeyCode, modifiers: Modifiers) -> Self {
		Self { code, kind: KeyEventKind::Press, modifiers, state: KeyEventState::empty() }
	}
}

impl From<KeyCode> for KeyEvent {
	fn from(code: KeyCode) -> Self {
		Self {
			code,
			kind: KeyEventKind::Press,
			modifiers: Modifiers::empty(),
			state: KeyEventState::empty(),
		}
	}
}

// --- Kind
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum KeyEventKind {
	#[default]
	Press,
	Release,
	Repeat,
}

impl KeyEventKind {
	pub(crate) fn from_vt_code(code: u8) -> Self {
		match code {
			2 => Self::Repeat,
			3 => Self::Release,
			_ => Self::Press,
		}
	}
}

// --- State
bitflags! {
	#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
	pub struct KeyEventState: u8 {
		const KEYPAD    = 1;
		const CAPS_LOCK = 2;
		const NUM_LOCK  = 4;
	}
}

impl KeyEventState {
	pub(crate) fn from_vt_mask(mask: u8) -> Self {
		let m = mask.saturating_sub(1);
		let mut state = Self::empty();
		if m & 64 != 0 {
			state |= Self::CAPS_LOCK;
		}
		if m & 128 != 0 {
			state |= Self::NUM_LOCK;
		}
		state
	}
}

// --- Code
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KeyCode {
	Char(char),
	Enter,
	Backspace,
	Tab,
	Escape,
	Left,
	Right,
	Up,
	Down,
	Home,
	End,
	PageUp,
	PageDown,
	Insert,
	Delete,
	KeypadBegin,
	CapsLock,
	ScrollLock,
	NumLock,
	PrintScreen,
	Pause,
	Menu,
	Null,
	Fn(u8),
	Modifier(ModifierKeyCode),
	Media(MediaKeyCode),
}

impl KeyCode {
	pub(crate) fn from_xterm_modifier(r#final: u8) -> Result<Self> {
		Ok(match r#final {
			b'A' => KeyCode::Up,
			b'B' => KeyCode::Down,
			b'C' => KeyCode::Right,
			b'D' => KeyCode::Left,
			b'F' => KeyCode::End,
			b'H' => KeyCode::Home,
			b'P' => KeyCode::Fn(1),
			b'Q' => KeyCode::Fn(2),
			b'R' => KeyCode::Fn(3),
			b'S' => KeyCode::Fn(4),
			_ => bail!(),
		})
	}

	pub(crate) fn from_codepoint(codepoint: u32) -> Result<(Self, KeyEventState)> {
		if let Ok(pair) = Self::from_kitty_codepoint(codepoint) {
			return Ok(pair);
		}

		let code = match char::from_u32(codepoint).ok_or(ParseError::Invalid)? {
			'\x1B' => Self::Escape,
			'\r' => Self::Enter,
			'\t' => Self::Tab,
			'\x7F' => Self::Backspace,
			c => Self::Char(c),
		};
		Ok((code, KeyEventState::empty()))
	}

	fn from_kitty_codepoint(codepoint: u32) -> Result<(Self, KeyEventState)> {
		// Keypad keys
		if (57399..=57427).contains(&codepoint) {
			let code = match codepoint {
				57399 => Self::Char('0'),
				57400 => Self::Char('1'),
				57401 => Self::Char('2'),
				57402 => Self::Char('3'),
				57403 => Self::Char('4'),
				57404 => Self::Char('5'),
				57405 => Self::Char('6'),
				57406 => Self::Char('7'),
				57407 => Self::Char('8'),
				57408 => Self::Char('9'),
				57409 => Self::Char('.'),
				57410 => Self::Char('/'),
				57411 => Self::Char('*'),
				57412 => Self::Char('-'),
				57413 => Self::Char('+'),
				57414 => Self::Enter,
				57415 => Self::Char('='),
				57416 => Self::Char(','),
				57417 => Self::Left,
				57418 => Self::Right,
				57419 => Self::Up,
				57420 => Self::Down,
				57421 => Self::PageUp,
				57422 => Self::PageDown,
				57423 => Self::Home,
				57424 => Self::End,
				57425 => Self::Insert,
				57426 => Self::Delete,
				57427 => Self::KeypadBegin,
				_ => unreachable!(),
			};
			return Ok((code, KeyEventState::KEYPAD));
		}

		// F13–F35 keys
		if (57376..=57398).contains(&codepoint) {
			return Ok((Self::Fn((codepoint - 57363) as u8), KeyEventState::empty()));
		}

		let code = match codepoint {
			57358 => Self::CapsLock,
			57359 => Self::ScrollLock,
			57360 => Self::NumLock,
			57361 => Self::PrintScreen,
			57362 => Self::Pause,
			57363 => Self::Menu,
			57428 => Self::Media(MediaKeyCode::Play),
			57429 => Self::Media(MediaKeyCode::Pause),
			57430 => Self::Media(MediaKeyCode::PlayPause),
			57431 => Self::Media(MediaKeyCode::Reverse),
			57432 => Self::Media(MediaKeyCode::Stop),
			57433 => Self::Media(MediaKeyCode::FastForward),
			57434 => Self::Media(MediaKeyCode::Rewind),
			57435 => Self::Media(MediaKeyCode::NextTrack),
			57436 => Self::Media(MediaKeyCode::PreviousTrack),
			57437 => Self::Media(MediaKeyCode::Record),
			57438 => Self::Media(MediaKeyCode::LowerVolume),
			57439 => Self::Media(MediaKeyCode::RaiseVolume),
			57440 => Self::Media(MediaKeyCode::MuteVolume),
			57441 => Self::Modifier(ModifierKeyCode::LeftShift),
			57442 => Self::Modifier(ModifierKeyCode::LeftControl),
			57443 => Self::Modifier(ModifierKeyCode::LeftAlt),
			57444 => Self::Modifier(ModifierKeyCode::LeftSuper),
			57445 => Self::Modifier(ModifierKeyCode::LeftHyper),
			57446 => Self::Modifier(ModifierKeyCode::LeftMeta),
			57447 => Self::Modifier(ModifierKeyCode::RightShift),
			57448 => Self::Modifier(ModifierKeyCode::RightControl),
			57449 => Self::Modifier(ModifierKeyCode::RightAlt),
			57450 => Self::Modifier(ModifierKeyCode::RightSuper),
			57451 => Self::Modifier(ModifierKeyCode::RightHyper),
			57452 => Self::Modifier(ModifierKeyCode::RightMeta),
			57453 => Self::Modifier(ModifierKeyCode::IsoLevel3Shift),
			57454 => Self::Modifier(ModifierKeyCode::IsoLevel5Shift),
			_ => bail!(),
		};
		Ok((code, KeyEventState::empty()))
	}
}

// --- Modifier key
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ModifierKeyCode {
	LeftShift,
	LeftControl,
	LeftAlt,
	LeftSuper, // Command/Windows/Super key
	LeftHyper,
	LeftMeta,
	RightShift,
	RightControl,
	RightAlt,
	RightSuper, // Command/Windows/Super key
	RightHyper,
	RightMeta,
	IsoLevel3Shift,
	IsoLevel5Shift,
}

impl ModifierKeyCode {
	pub(crate) fn to_modifier(self) -> Option<Modifiers> {
		match self {
			Self::LeftShift | Self::RightShift => Some(Modifiers::SHIFT),
			Self::LeftControl | Self::RightControl => Some(Modifiers::CONTROL),
			Self::LeftAlt | Self::RightAlt => Some(Modifiers::ALT),
			Self::LeftSuper | Self::RightSuper => Some(Modifiers::SUPER),
			Self::LeftHyper | Self::RightHyper => Some(Modifiers::HYPER),
			Self::LeftMeta | Self::RightMeta => Some(Modifiers::META),
			_ => None,
		}
	}
}

// --- Media key
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MediaKeyCode {
	Play,
	Pause,
	PlayPause,
	Reverse,
	Stop,
	FastForward,
	Rewind,
	NextTrack,
	PreviousTrack,
	Record,
	LowerVolume,
	RaiseVolume,
	MuteVolume,
}
