use std::sync::Arc;

use yazi_macro::impl_data_any;
use yazi_shared::url::UrlBuf;

#[derive(Clone, Debug)]
pub struct View {
	pub url:     UrlBuf,
	pub focused: bool,
	pub offset:  usize,
	text:        Arc<Text>,
}

#[derive(Debug)]
struct Text {
	body:  String,
	lines: Vec<usize>,
}

impl_data_any!(View);

impl View {
	pub fn new(url: UrlBuf, body: String) -> Option<Self> {
		let mut lines = vec![0];
		for (index, byte) in body.bytes().enumerate() {
			match byte {
				b'\n' => lines.push(index + 1),
				b'\r' | b'\t' => {}
				0..=31 | 127 => return None,
				_ => {}
			}
		}

		Some(Self { url, focused: true, offset: 0, text: Arc::new(Text { body, lines }) })
	}

	pub fn last_offset(&self, height: usize) -> usize {
		self.text.lines.len().saturating_sub(height.max(1))
	}

	pub fn visible(&self, height: usize) -> impl Iterator<Item = &str> {
		let body = &self.text.body;
		let lines = &self.text.lines[self.offset.min(self.last_offset(height))..];

		lines.iter().take(height).enumerate().map(move |(i, &start)| {
			let end = lines.get(i + 1).copied().unwrap_or(body.len());
			let line = body[start..end].trim_end_matches(['\r', '\n']);
			if start == 0 { line.strip_prefix('\u{feff}').unwrap_or(line) } else { line }
		})
	}
}
