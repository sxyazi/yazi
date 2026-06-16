use base64::Engine;
use yazi_shim::BASE64_PAD;

use crate::{ParseError, Result, parser::{Osc5522Status, Parser, State}};

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

	pub(super) fn parse_osc5522(&mut self) -> Result<()> {
		debug_assert!(self.seq.starts_with(b"\x1b]5522;"));
		debug_assert!(self.seq.ends_with(b"\x1b\\"));

		let mut it = self.seq[7..self.seq.len() - 2].splitn(2, |&b| b == b';');
		let meta = str::from_utf8(it.next().ok_or(ParseError::Invalid)?)?;
		let payload = it.next().unwrap_or(&[]);

		let State::Osc5522(state) = &mut self.state else { unreachable!() };
		state.has_more = false;

		for part in meta.split(':') {
			match part.split_once('=').ok_or(ParseError::Invalid)? {
				("status", v) => match v {
					"OK" => {
						state.status = Some(Osc5522Status::OK);
						state.has_more = true;
					}
					"DATA" => {
						state.status = Some(Osc5522Status::DATA);
						state.has_more = true;
					}
					"DONE" => state.status = Some(Osc5522Status::DONE),
					"ENOSYS" => state.status = Some(Osc5522Status::ENOSYS),
					"EPERM" => state.status = Some(Osc5522Status::EPERM),
					"EBUSY" => state.status = Some(Osc5522Status::EBUSY),
					"EIO" => state.status = Some(Osc5522Status::EIO),
					"EINVAL" => state.status = Some(Osc5522Status::EINVAL),
					_ => return Err(ParseError::Invalid),
				},
				("type", v) => state.read = v == "read",
				("loc", v) => state.primary = v == "primary",
				("mime", v) => {
					let bytes = BASE64_PAD.decode(v.as_bytes()).or(Err(ParseError::Invalid))?;
					if state.mime.len() == 0 {
						state.mime.push(bytes);
					} else if state.mime[state.idx] != bytes {
						state.mime.push(bytes);
						state.idx += 1;
					}
				}
				("pw", v) => state.pw = BASE64_PAD.decode(v.as_bytes()).unwrap_or_default(),
				_ => {}
			}
		}

		// decode now since each payload may have its own padding
		let payload = BASE64_PAD.decode(&payload).or(Err(ParseError::Invalid))?;

		if state.idx >= state.payload.len() {
			state.payload.push(payload.to_vec());
		} else {
			state.payload[state.idx].extend(payload);
		}

		// Limit payload size to 1MiB per mime type to prevent potential DoS
		// TODO A larger size would be required for directly pasting images/large files
		if state.payload[state.idx].len() > 1 << 20 {
			return Err(ParseError::Invalid);
		}

		Ok(())
	}
}
