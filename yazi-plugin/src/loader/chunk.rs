use std::borrow::Cow;

use mlua::{AsChunk, ChunkMode};
use yazi_shared::natsort;

pub struct Chunk {
	pub mode:       ChunkMode,
	pub bytes:      Cow<'static, [u8]>,
	pub since:      String,
	pub sync_peek:  bool,
	pub sync_entry: bool,
}

impl Chunk {
	#[inline]
	pub fn compatible(&self) -> bool {
		let s = yazi_boot::actions::Actions::version();
		natsort(s.as_bytes(), self.since.as_bytes(), false) != std::cmp::Ordering::Less
	}

	fn analyze(&mut self) {
		for line in self.bytes.split(|&b| b == b'\n') {
			if line.trim_ascii().is_empty() {
				continue;
			};

			let Some(rest) = line.strip_prefix(b"---") else { break };
			let rest = rest.trim_ascii();

			let Some(i) = rest.iter().position(|&b| b == b' ' || b == b'\t') else { break };
			match (rest[..i].trim_ascii(), rest[i..].trim_ascii()) {
				(b"@sync", b"peek") => self.sync_peek = true,
				(b"@sync", b"entry") => self.sync_entry = true,

				(b"@since", b"") => continue,
				(b"@since", b) => self.since = String::from_utf8_lossy(b).to_string(),

				(_, []) => break,
				(b, _) if b.strip_prefix(b"@").unwrap_or(b"").is_empty() => break,
				_ => continue,
			}
		}
	}
}

impl From<Cow<'static, [u8]>> for Chunk {
	fn from(b: Cow<'static, [u8]>) -> Self {
		let mut chunk = Self {
			mode:       ChunkMode::Text,
			bytes:      b,
			since:      String::new(),
			sync_entry: false,
			sync_peek:  false,
		};
		chunk.analyze();
		chunk
	}
}

impl From<&'static [u8]> for Chunk {
	fn from(b: &'static [u8]) -> Self { Self::from(Cow::Borrowed(b)) }
}

impl From<Vec<u8>> for Chunk {
	fn from(b: Vec<u8>) -> Self { Self::from(Cow::Owned(b)) }
}

impl<'a> AsChunk<'a> for &'a Chunk {
	fn source(self) -> std::io::Result<Cow<'a, [u8]>> { Ok(Cow::Borrowed(&self.bytes)) }

	fn mode(&self) -> Option<ChunkMode> { Some(self.mode) }
}
