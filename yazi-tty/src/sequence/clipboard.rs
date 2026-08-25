use std::fmt::{self, Display};

use base64::{display::Base64Display, engine::general_purpose::{self, STANDARD_PAD_INDIFFERENT}};

use super::traits::{ListMimes, Mimelist};

/// Set clipboard contents via OSC 52.
pub struct SetClipboard<'a>(pub &'a [u8]);

impl Display for SetClipboard<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b]52;c;{}\x1b\\", Base64Display::new(self.0, &general_purpose::STANDARD))
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
pub struct ReadClipboard<'a, M> {
	pub(crate) mimes:   M,
	pub(crate) pw:      &'a str,
	pub(crate) name:    &'a str,
	pub(crate) primary: bool,
}

impl<M: Mimelist> Display for ReadClipboard<'_, M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b]5522;type=read")?;
		if !self.pw.is_empty() {
			write!(
				f,
				":pw={}:name={}",
				Base64Display::new(self.pw.as_bytes(), &STANDARD_PAD_INDIFFERENT),
				Base64Display::new(self.name.as_bytes(), &STANDARD_PAD_INDIFFERENT)
			)?;
		}

		if self.primary {
			write!(f, ":loc=primary")?;
		}

		let mimes = ListMimes(self.mimes.clone())
			.encode_base64(&STANDARD_PAD_INDIFFERENT)
			.map_err(|_| fmt::Error)?;
		write!(f, ";{mimes}\x1b\\")
	}
}

/// Write a complete OSC 5522 clipboard transmission.
pub struct WriteClipboard<'a, M> {
	data: Vec<WriteClipboardData<'a, M>>,
}

impl<M: Mimelist> Display for WriteClipboard<'_, M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{WriteClipboardHead}")?;
		for data in &self.data {
			write!(f, "{data}")?;
		}
		write!(f, "{WriteClipboardTail}")
	}
}

/// Begin an OSC 5522 clipboard transmission: `OSC 5522 ; type=write ST`.
pub struct WriteClipboardHead;

impl Display for WriteClipboardHead {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b]5522;type=write\x1b\\")
	}
}

/// Write one MIME payload and its aliases.
///
/// The payload is sent as one or more
/// `OSC 5522 ; type=wdata:mime=<base64 MIME type> ; <base64 data chunk> ST`
/// packets, followed by an optional `type=walias` packet.
pub struct WriteClipboardData<'a, M> {
	pub(crate) mime:    &'a str,
	pub(crate) payload: &'a [u8],
	pub(crate) aliases: M,
}

impl<M: Mimelist> Display for WriteClipboardData<'_, M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mime = Base64Display::new(self.mime.as_bytes(), &STANDARD_PAD_INDIFFERENT);
		for chunk in self.payload.chunks(4096).chain(self.payload.is_empty().then_some(&[] as &[u8])) {
			write!(
				f,
				"\x1b]5522;type=wdata:mime={mime};{}\x1b\\",
				Base64Display::new(chunk, &STANDARD_PAD_INDIFFERENT)
			)?;
		}

		let aliases = ListMimes(self.aliases.clone())
			.encode_base64(&STANDARD_PAD_INDIFFERENT)
			.map_err(|_| fmt::Error)?;
		if !aliases.is_empty() {
			write!(f, "\x1b]5522;type=walias:mime={mime};{}\x1b\\", aliases)?;
		}
		Ok(())
	}
}

/// Finish an OSC 5522 clipboard transmission: `OSC 5522 ; type=wdata ST`.
pub struct WriteClipboardTail;

impl Display for WriteClipboardTail {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\x1b]5522;type=wdata\x1b\\")
	}
}
