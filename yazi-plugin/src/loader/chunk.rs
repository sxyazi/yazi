use std::borrow::Cow;

pub struct Chunk {
	pub bytes:      Cow<'static, [u8]>,
	pub sync_entry: bool,
}

impl Chunk {
	#[inline]
	pub fn as_bytes(&self) -> &[u8] { &self.bytes }

	fn analyze(&mut self) {
		for line in self.bytes.split(|&b| b == b'\n') {
			let Some(rest) = line.strip_prefix(b"---") else { break };

			let rest = rest.trim_ascii();
			let Some(pos) = rest.iter().position(|&b| b == b' ' || b == b'\t') else { break };

			match (rest[..pos].trim_ascii(), rest[pos..].trim_ascii()) {
				(b"@sync", b"entry") => self.sync_entry = true,
				(_, []) => break,
				(b, _) if b.strip_prefix(b"@").unwrap_or(b"").is_empty() => break,
				_ => continue,
			}
		}
	}
}

impl From<Cow<'static, [u8]>> for Chunk {
	fn from(b: Cow<'static, [u8]>) -> Self {
		let mut chunk = Self { bytes: b, sync_entry: false };
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
