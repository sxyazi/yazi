use std::fmt;

use anyhow::{Result, bail, ensure};

const ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvABCDEFGHIJKLMNOPQRSTUVWXYZ-._~!$";

pub(super) struct Varint;

impl Varint {
	pub(super) fn write_u64(writer: &mut impl fmt::Write, mut value: u64) -> fmt::Result {
		loop {
			let more = value > 0b11111;
			writer.write_char(ALPHABET[((value & 0b11111) as usize) + more as usize * 32] as char)?;

			if !more {
				return Ok(());
			}
			value >>= 5;
		}
	}

	pub(super) fn write_i64(writer: &mut impl fmt::Write, value: i64) -> fmt::Result {
		// ZigZag encoding: https://protobuf.dev/programming-guides/encoding/#signed-integers
		Self::write_u64(writer, ((value as u64) << 1) ^ (value >> 63) as u64)
	}

	pub(super) fn read_u64(bytes: &[u8]) -> Result<(u64, usize)> {
		let mut value = 0;
		for (i, &byte) in bytes.iter().enumerate() {
			let (digit, more) = match byte {
				b'0'..=b'9' => (byte - b'0', false),
				b'a'..=b'v' => (byte - b'a' + 10, false),
				b'A'..=b'Z' => (byte - b'A', true),
				b'-' => (26, true),
				b'.' => (27, true),
				b'_' => (28, true),
				b'~' => (29, true),
				b'!' => (30, true),
				b'$' => (31, true),
				_ => bail!("invalid varint digit"),
			};

			let shift = i * 5;
			ensure!(shift < 64 && digit as u64 <= u64::MAX >> shift, "varint overflow");

			value |= u64::from(digit) << shift;
			if !more {
				return Ok((value, i + 1));
			}
		}

		bail!("unexpected end of varint")
	}

	pub(super) fn read_i64(bytes: &[u8]) -> Result<(i64, usize)> {
		let (value, len) = Self::read_u64(bytes)?;
		Ok((((value >> 1) as i64) ^ -((value & 1) as i64), len))
	}
}
