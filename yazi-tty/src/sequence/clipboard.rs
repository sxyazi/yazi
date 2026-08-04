use std::fmt::{self, Display};

use base64::{Engine, engine::general_purpose::{self, STANDARD_PAD_INDIFFERENT}};

/// Set clipboard contents via OSC 52.
pub struct SetClipboard {
	content: String,
}

impl SetClipboard {
	pub fn new(content: impl AsRef<[u8]>) -> Self {
		Self { content: general_purpose::STANDARD.encode(content) }
	}
}

impl Display for SetClipboard {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b]52;c;{}\x1b\\", self.content)
	}
}

/// Probe terminal clipboard support.
pub struct ProbeClipboard;

impl Display for ProbeClipboard {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "\x1b[?5522$p") }
}

/// Enable clipboard support: `CSI ? 5522 h`.
pub struct EnableClipboard;

impl Display for EnableClipboard {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "\x1b[?5522h") }
}

/// Disable clipboard support: `CSI ? 5522 l`.
pub struct DisableClipboard;

impl Display for DisableClipboard {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "\x1b[?5522l") }
}

/// Request clipboard data for the given MIME types.
/// `OSC 5522 ; type=read[:metadata] ; <base64 MIME list> ST`
pub struct ReadClipboard<'a> {
	pub mimes:   &'a [u8],
	pub pw:      &'a str,
	pub name:    &'a str,
	pub primary: bool,
}

impl Display for ReadClipboard<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut metadata = String::new();
		if self.pw.len() > 0 {
			let b64_pw = STANDARD_PAD_INDIFFERENT.encode(self.pw);
			let b64_name = STANDARD_PAD_INDIFFERENT.encode(self.name);
			metadata.push_str(&format!(":pw={}:name={}", b64_pw, b64_name));
		}
		if self.primary {
			metadata.push_str(":loc=primary");
		}
		write!(
			f,
			"\x1b]5522;type=read{};{}\x1b\\",
			metadata,
			STANDARD_PAD_INDIFFERENT.encode(self.mimes)
		)
	}
}

/// Write clipboard data.
/// `OSC 5522 ; type=write ST`
/// `OSC 5522 ; type=wdata : mime=<base64 MIME type> ; <base64 data chunk> ST`
/// `OSC 5522 ; type=wdata ST`
pub struct WriteClipboard<'a> {
	pub data: Vec<WriteClipboardData<'a>>,
}

impl Display for WriteClipboard<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b]5522;type=write\x1b\\")?;
		for item in &self.data {
			let b64_mime = STANDARD_PAD_INDIFFERENT.encode(item.mime);
			let data = item.payload;

			for chunk in data.chunks(4096).chain(data.is_empty().then_some(&[] as &[u8])) {
				let b64_chunk = STANDARD_PAD_INDIFFERENT.encode(chunk);
				write!(f, "\x1b]5522;type=wdata:mime={};{}\x1b\\", b64_mime, b64_chunk)?;
			}

			if item.alias.len() > 0 {
				let b64_alias = STANDARD_PAD_INDIFFERENT.encode(item.alias);
				write!(f, "\x1b]5522;type=walias:mime={};{}\x1b\\", b64_mime, b64_alias)?;
			}
		}
		write!(f, "\x1b]5522;type=wdata\x1b\\")
	}
}

/// A MIME payload written by [`WriteClipboard`].
pub struct WriteClipboardData<'a> {
	pub mime:    &'a [u8],
	pub payload: &'a [u8],
	pub alias:   &'a [u8],
}
