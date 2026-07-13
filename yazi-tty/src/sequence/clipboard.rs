use std::fmt::{self, Display};

use base64::{Engine, engine::general_purpose};
use yazi_shim::BASE64_PAD;

/// Set clipboard content via OSC 52
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

/// Query OSC 5522 via DECRQM
pub struct QueryOSC5522;

impl Display for QueryOSC5522 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "\x1b[?5522$p") }
}

/// Enable receiving unsolicited paste events via OSC 5522: `CSI ? 5522 h`
pub struct EnablePasteEvents;

impl Display for EnablePasteEvents {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "\x1b[?5522h") }
}

/// Disable receiving unsolicited paste events via OSC 5522: `CSI ? 5522 l`
pub struct DisablePasteEvents;

impl Display for DisablePasteEvents {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "\x1b[?5522l") }
}

/// Read data from clipboard:
/// `OSC 5522 ; type=read : <metadata> ; <base64 MIME list> ST`
pub struct ReadClipboard<'a> {
	pub mime:    &'a [u8],
	pub pw:      &'a [u8],
	pub name:    &'a [u8],
	pub primary: bool,
}

impl Display for ReadClipboard<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let b64_mime = BASE64_PAD.encode(self.mime);
		let mut metadata = String::new();
		if self.pw.len() > 0 {
			let b64_pw = BASE64_PAD.encode(self.pw);
			let b64_name = BASE64_PAD.encode(self.name);
			metadata.push_str(&format!(":pw={}:name={}", b64_pw, b64_name));
		}
		if self.primary {
			metadata.push_str(":loc=primary");
		}
		write!(f, "\x1b]5522;type=read{};{}\x1b\\", metadata, b64_mime)
	}
}

/// Read available MIME types from clipboard:
/// `OSC 5522 ; type=read ; <base64 [.]> ST`
pub struct ReadClipboardMimes;

impl Display for ReadClipboardMimes {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b]5522;type=read;{}\x1b\\", BASE64_PAD.encode(b"."))
	}
}

/// Write data to clipboard:
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
			let b64_mime = BASE64_PAD.encode(item.mime);
			let data = item.payload;

			for (_, chunk) in data.chunks(4096).enumerate() {
				let b64_chunk = BASE64_PAD.encode(chunk);
				write!(f, "\x1b]5522;type=wdata:mime={};{}\x1b\\", b64_mime, b64_chunk)?;
			}

			if item.alias.len() > 0 {
				let b64_alias = BASE64_PAD.encode(item.alias);
				write!(f, "\x1b]5522;type=walias:mime={};{}\x1b\\", b64_mime, b64_alias)?;
			}
		}
		write!(f, "\x1b]5522;type=wdata\x1b\\")
	}
}

pub struct WriteClipboardData<'a> {
	pub mime:    &'a [u8],
	pub payload: &'a [u8],
	pub alias:   &'a [u8],
}
