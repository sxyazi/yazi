use crate::{ParseError, Result, parser::{Parser, State}};

impl Parser {
	pub(super) fn parse_osc72(&mut self) -> Result<()> {
		debug_assert!(self.seq.starts_with(b"\x1b]72;"));
		debug_assert!(self.seq.ends_with(b"\x1b\\"));

		let mut it = self.seq[5..self.seq.len() - 2].splitn(2, |&b| b == b';');
		let meta = str::from_utf8(it.next().ok_or(ParseError::Invalid)?)?;
		let payload = it.next().unwrap_or(&[]);

		let State::Osc72(state) = &mut self.state else { unreachable!() };
		state.has_more = false; // reset has_more for this new sequence

		for part in meta.split(':') {
			match part.split_once('=').ok_or(ParseError::Invalid)? {
				("t", v) if let Some(b) = v.bytes().next() => state.r#type = Some(b),
				("x", v) => state.x = Some(v.parse()?),
				("y", v) => state.y = Some(v.parse()?),
				("o", v) => state.op = Some(v.parse()?),
				("m", "1") => state.has_more = true,
				_ => {}
			}
		}

		// For `t=r`, the presence of a payload indicates more data is coming,
		// even if `m=1` is not set.
		if state.r#type == Some(b'r') && !payload.is_empty() {
			state.has_more = true;
		}

		// Limit payload size to 1MiB to prevent potential DoS
		if state.payload.len() + payload.len() > 1 << 20 {
			return Err(ParseError::Invalid);
		}

		state.payload.extend(payload);
		Ok(())
	}
}
