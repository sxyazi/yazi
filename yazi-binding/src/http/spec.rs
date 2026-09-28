use reqwest::header::HeaderValue;
use tokio::sync::mpsc;

use super::{HttpForm, HttpPart};

pub(crate) enum HttpSpec {
	Bytes(Vec<u8>),
	Raw { tx: Option<mpsc::Sender<Vec<u8>>>, rx: mpsc::Receiver<Vec<u8>> },
	UrlEncoded(HttpForm),
	Multipart(HttpForm),
}

impl Default for HttpSpec {
	fn default() -> Self { Self::Bytes(Vec::new()) }
}

impl HttpSpec {
	pub(super) fn bytes(&mut self, bytes: Vec<u8>) { *self = Self::Bytes(bytes); }

	pub(super) fn raw(&mut self) {
		let (tx, rx) = mpsc::channel(1);
		*self = Self::Raw { tx: Some(tx), rx };
	}

	pub(super) fn field(&mut self, name: String, value: String) {
		if !matches!(self, Self::UrlEncoded(_) | Self::Multipart(_)) {
			*self = Self::UrlEncoded(HttpForm::default());
		}

		if let Self::UrlEncoded(form) | Self::Multipart(form) = self {
			form.field(name, value);
		}
	}

	pub(super) fn part(&mut self, part: HttpPart) -> reqwest::Result<()> {
		if let Self::UrlEncoded(form) = self {
			*self = Self::Multipart(std::mem::take(form));
		} else if !matches!(self, Self::Multipart(_)) {
			*self = Self::Multipart(HttpForm::default());
		}

		if let Self::Multipart(form) = self {
			form.part(part)?;
		}
		Ok(())
	}

	pub(super) fn content_type(&self) -> Option<HeaderValue> {
		Some(match self {
			Self::UrlEncoded(_) => HeaderValue::from_static("application/x-www-form-urlencoded"),
			Self::Multipart(form) => {
				HeaderValue::from_str(&format!("multipart/form-data; boundary={}", form.boundary()))
					.expect("boundary is always valid")
			}
			_ => return None,
		})
	}

	pub(super) fn take_tx(&mut self) -> Option<mpsc::Sender<Vec<u8>>> {
		match self {
			Self::Raw { tx, .. } => tx.take(),
			_ => None,
		}
	}
}
