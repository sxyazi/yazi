use std::str::{self, FromStr};

use super::parser::Parser;
use crate::{ParseError, Result, bail, event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, Modifiers, MouseEvent, MouseEventKind}};

impl Parser {
	pub(super) fn parse_csi(&self) -> Result<Event> {
		let seq = &self.seq;
		debug_assert!(seq.starts_with(b"\x1B["));

		if seq.len() == 2 {
			return Err(ParseError::Incomplete);
		}

		let first = seq[2];
		let second = seq.get(3);
		let last = seq[seq.len() - 1];

		Ok(match first {
			b'[' => match second {
				None => return Err(ParseError::Incomplete),
				Some(b @ b'A'..=b'E') => Event::Key(KeyCode::Fn(1 + b - b'A').into()),
				Some(_) => bail!(),
			},
			b'D' => Event::Key(KeyCode::Left.into()),
			b'C' => Event::Key(KeyCode::Right.into()),
			b'A' => Event::Key(KeyCode::Up.into()),
			b'B' => Event::Key(KeyCode::Down.into()),
			b'H' => Event::Key(KeyCode::Home.into()),
			b'F' => Event::Key(KeyCode::End.into()),
			b'Z' => Event::Key(KeyEvent::new(KeyCode::Tab, Modifiers::SHIFT)),
			b'M' => return self.parse_csi_normal_mouse(),
			b'<' => return self.parse_csi_sgr_mouse(),
			b'I' => Event::FocusIn,
			b'O' => Event::FocusOut,
			b';' => return self.parse_csi_modifier_key(),
			// P, Q, and S for compatibility with Kitty keyboard protocol,
			// as the 1 in 'CSI 1 P' etc. must be omitted if there are no
			// modifiers pressed:
			// https://sw.kovidgoyal.net/kitty/keyboard-protocol/#legacy-functional-keys
			b'P' => Event::Key(KeyCode::Fn(1).into()),
			b'Q' => Event::Key(KeyCode::Fn(2).into()),
			b'S' => Event::Key(KeyCode::Fn(4).into()),
			b'?' => match last {
				b'u' | b'c' | b'n' | b'y' => return Err(ParseError::Ignored),
				_ => bail!(),
			},
			b'>' => match seq[seq.len() - 2..] {
				[b' ', b'q'] => return Err(ParseError::Ignored),
				_ => bail!(),
			},
			b'0'..=b'9' if !(64..=126).contains(&last) => return Err(ParseError::Incomplete),
			b'0'..=b'9' => match last {
				b'M' => return self.parse_csi_rxvt_mouse(),
				b'~' => return self.parse_csi_special_key(),
				b'u' => return self.parse_csi_u_key(),
				b'R' => return Err(ParseError::Ignored),
				_ if self.seq.contains(&b';') => return self.parse_csi_modifier_key(),
				_ => return self.parse_csi_modifier_legacy_key(),
			},
			_ => bail!(),
		})
	}

	pub(super) fn parse_csi_u_key(&self) -> Result<Event> {
		let seq = &self.seq;
		debug_assert!(seq.starts_with(b"\x1B[")); // CSI
		debug_assert!(seq.ends_with(b"u"));

		let s = str::from_utf8(&seq[2..seq.len() - 1])?;
		let mut it = s.split(';');

		// In `CSI u`, this is parsed as:
		//
		//     CSI codepoint ; modifiers u
		//     codepoint: ASCII Dec value
		//
		// The Kitty Keyboard Protocol extends this with optional components that can be
		// enabled progressively. The full sequence is parsed as:
		//
		//     CSI unicode-key-code:alternate-key-codes ; modifiers:event-type ;
		// text-as-codepoints u
		let mut codepoints = it.next().ok_or(ParseError::Invalid)?.split(':');

		let (mut code, state_from_keycode) = KeyCode::from_codepoint(parse_next(&mut codepoints)?)?;
		let (mut modifiers, kind, state_from_modifiers) = parse_mks(&mut it).unwrap_or_default();

		if let KeyCode::Modifier(c) = code
			&& let Some(m) = c.to_modifier()
		{
			modifiers |= m;
		}

		// When the "report alternate keys" flag is enabled in the Kitty Keyboard
		// Protocol and the terminal sends a keyboard event containing shift, the
		// sequence will contain an additional codepoint separated by a ':' character
		// which contains the shifted character according to the keyboard layout.
		if modifiers.contains(Modifiers::SHIFT)
			&& let Ok(Some(shifted)) = parse_next(&mut codepoints).map(char::from_u32)
		{
			code = KeyCode::Char(shifted);
			modifiers.remove(Modifiers::SHIFT);
		}

		Ok(Event::Key(KeyEvent {
			code,
			modifiers,
			kind,
			state: state_from_keycode | state_from_modifiers,
		}))
	}

	/// Parses `CSI [1;] modifier[:kind] final` — sequences that carry a
	/// semicolon, e.g. `\x1B[;2A` (Shift+Up, leading 1 omitted) or `\x1B[1;2A`
	/// (Shift+Up).
	pub(super) fn parse_csi_modifier_key(&self) -> Result<Event> {
		let seq = &self.seq;
		debug_assert!(seq.starts_with(b"\x1B[")); // CSI

		let s = str::from_utf8(&seq[2..seq.len() - 1])?;
		let mut it = s.split(';');
		it.next(); // skip leading "1" or empty string

		let (modifiers, kind, _) = parse_mks(&mut it).unwrap_or_default();
		let code = KeyCode::from_xterm_modifier(seq[seq.len() - 1])?;
		Ok(Event::Key(KeyEvent { code, modifiers, kind, state: KeyEventState::empty() }))
	}

	/// Parses legacy `CSI modifier final` - no semicolon, modifier digit
	/// immediately before the final byte, e.g. `\x1B[2A` = Shift+Up.
	pub(super) fn parse_csi_modifier_legacy_key(&self) -> Result<Event> {
		let seq = &self.seq;
		debug_assert!(seq.starts_with(b"\x1B[")); // CSI

		let modifier = seq[seq.len() - 2];
		if !modifier.is_ascii_digit() {
			bail!();
		}

		Ok(Event::Key(KeyEvent {
			code:      KeyCode::from_xterm_modifier(seq[seq.len() - 1])?,
			modifiers: Modifiers::from_vt_mask(modifier - b'0'),
			kind:      KeyEventKind::Press,
			state:     KeyEventState::empty(),
		}))
	}

	pub(super) fn parse_csi_special_key(&self) -> Result<Event> {
		let seq = &self.seq;
		debug_assert!(seq.starts_with(b"\x1B[")); // CSI
		debug_assert!(seq.ends_with(b"~"));

		let s = str::from_utf8(&seq[2..seq.len() - 1])?;
		let mut it = s.split(';');

		// This CSI sequence can be a list of semicolon-separated numbers.
		let first: u8 = parse_next(&mut it)?;

		let (modifiers, kind, state) = parse_mks(&mut it).unwrap_or_default();

		let code = match first {
			1 | 7 => KeyCode::Home,
			2 => KeyCode::Insert,
			3 => KeyCode::Delete,
			4 | 8 => KeyCode::End,
			5 => KeyCode::PageUp,
			6 => KeyCode::PageDown,
			v @ 11..=15 => KeyCode::Fn(v - 10),
			v @ 17..=21 => KeyCode::Fn(v - 11),
			v @ 23..=26 => KeyCode::Fn(v - 12),
			v @ 28..=29 => KeyCode::Fn(v - 15),
			v @ 31..=34 => KeyCode::Fn(v - 17),
			_ => bail!(),
		};

		let event = Event::Key(KeyEvent { code, modifiers, kind, state });

		Ok(event)
	}

	// Parse rxvt mouse: CSI Cb ; Cx ; Cy ; M
	pub(super) fn parse_csi_rxvt_mouse(&self) -> Result<Event> {
		let seq = &self.seq;
		debug_assert!(seq.starts_with(b"\x1B[")); // CSI
		debug_assert!(seq.ends_with(b"M"));

		let s = str::from_utf8(&seq[2..seq.len() - 1])?;
		let mut it = s.split(';');

		let cb = parse_next::<u8>(&mut it)?.checked_sub(32).ok_or(ParseError::Invalid)?;
		let (kind, modifiers) = MouseEventKind::from_cb(cb)?;

		let column = parse_next::<u16>(&mut it)?.checked_sub(1).ok_or(ParseError::Invalid)?;
		let row = parse_next::<u16>(&mut it)?.checked_sub(1).ok_or(ParseError::Invalid)?;

		Ok(Event::Mouse(MouseEvent { kind, column, row, modifiers }))
	}

	// Parse normal mouse: CSI M CB Cx Cy (6 characters only).
	pub(super) fn parse_csi_normal_mouse(&self) -> Result<Event> {
		let seq = &self.seq;
		debug_assert!(seq.starts_with(b"\x1B[M")); // CSI M

		if seq.len() < 6 {
			return Err(ParseError::Incomplete);
		}

		let cb = seq[3].checked_sub(32).ok_or(ParseError::Invalid)?;
		let (kind, modifiers) = MouseEventKind::from_cb(cb)?;

		// See http://www.xfree86.org/current/ctlseqs.html#Mouse%20Tracking
		// Mouse positions are encoded as (value + 32), but the upper left
		// character position on the terminal is denoted as 1,1.
		// So, we need to subtract 32 + 1 (33) to keep it synced with the cursor.
		let column = u16::from(seq[4].checked_sub(33).ok_or(ParseError::Invalid)?);
		let row = u16::from(seq[5].checked_sub(33).ok_or(ParseError::Invalid)?);

		Ok(Event::Mouse(MouseEvent { kind, column, row, modifiers }))
	}

	// Parse SGR mouse: CSI < Cb ; Cx ; Cy (;) (M or m)
	pub(super) fn parse_csi_sgr_mouse(&self) -> Result<Event> {
		let seq = &self.seq;
		debug_assert!(seq.starts_with(b"\x1B[<")); // CSI <

		if !seq.ends_with(b"m") && !seq.ends_with(b"M") {
			return Err(ParseError::Ignored);
		}

		let s = str::from_utf8(&seq[3..seq.len() - 1])?;
		let mut it = s.split(';');

		let cb = parse_next(&mut it)?;
		let (mut kind, modifiers) = MouseEventKind::from_cb(cb)?;

		// See http://www.xfree86.org/current/ctlseqs.html#Mouse%20Tracking
		// The upper left character position on the terminal is denoted as 1,1.
		// Subtract 1 to keep it synced with cursor
		let column = parse_next::<u16>(&mut it)?.checked_sub(1).ok_or(ParseError::Invalid)?;
		let row = parse_next::<u16>(&mut it)?.checked_sub(1).ok_or(ParseError::Invalid)?;

		// When button 3 in Cb is used to represent mouse release, you can't tell which
		// button was released. SGR mode solves this by having the sequence end with a
		// lowercase m if it's a button release and an uppercase M if it's a button
		// press.
		//
		// We've already checked that the last character is a lowercase or uppercase M
		// at the start of this function, so we just need one if.
		if seq.ends_with(b"m")
			&& let MouseEventKind::Down(button) = kind
		{
			kind = MouseEventKind::Up(button);
		}

		Ok(Event::Mouse(MouseEvent { kind, column, row, modifiers }))
	}
}

fn parse_next<'a, T>(iter: &mut impl Iterator<Item = &'a str>) -> Result<T>
where
	T: FromStr,
{
	iter.next().ok_or(ParseError::Invalid)?.parse::<T>().map_err(|_| ParseError::Invalid)
}

fn parse_mks<'a, I>(mut it: I) -> Option<(Modifiers, KeyEventKind, KeyEventState)>
where
	I: Iterator<Item = &'a str>,
{
	let mut it = it.next()?.split(':');
	let mask: u8 = it.next()?.parse().ok()?;
	let code: u8 = it.next().and_then(|s| s.parse().ok()).unwrap_or(1);

	Some((
		Modifiers::from_vt_mask(mask),
		KeyEventKind::from_vt_code(code),
		KeyEventState::from_vt_mask(mask),
	))
}
