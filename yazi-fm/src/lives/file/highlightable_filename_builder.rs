#![allow(unused_variables)]
#![allow(dead_code)]

use std::{collections::HashMap, ops::Range};

use yazi_shared::theme::Style;

/// Represents the name and extension of a file or directory, that can
/// optionally be highlighted (e.g. when searching, to highlight the matched
/// characters), and converted to `ui.Span` elements for rendering. Consecutive
/// highlights with the same style are merged into a single span.
pub struct HighlightedFilenameBuilder {
	pub(crate) stem:      String,
	/// The extension of the file, if any. Must have a leading dot.
	/// Example: `".txt"`
	pub(crate) extension: Option<String>,

	/// Maps the index of a character in the full text to its highlight style.
	pub(crate) highlights: HashMap<usize, Style>,
}

// NOTE: Since rust strings are utf-8 strings, the characters are not
// necessarily one byte long. "".chars().count() will return the number of
// characters in the string, not the number of bytes.
impl HighlightedFilenameBuilder {
	pub fn new(stem: String, extension_with_leading_dot: Option<String>) -> Self {
		assert!(
			extension_with_leading_dot.is_none()
				|| extension_with_leading_dot.as_ref().unwrap().starts_with('.')
		);
		Self { stem, extension: extension_with_leading_dot, highlights: HashMap::new() }
	}

	/// Adds a default color to the extension. It might be partially or fully
	/// overwritten by highlights that are added later.
	pub fn add_extension_highlight(&mut self, style: Style) {
		if let Some(extension) = &self.extension {
			let start = self.stem.len();
			let end = self.stem.len() + extension.len();

			self.add_highlight(start..end, style);
		}
	}

	/// Makes the indices specified by the given Range as having the given
	/// highlight style. Old highlights are overwritten.
	pub fn add_highlight(&mut self, range: Range<usize>, style: Style) {
		for i in range {
			self.highlights.insert(i, style);
		}
	}

	/// Consumes the builder and produces styled spans of text for rendering. The
	/// stem and extension are separated into different spans.
	pub fn build_spans(&mut self) -> Vec<HighlightedSpan> {
		let (stem_highlights, extension_highlights) = {
			let mut highlights = Vec::new();

			self.stem.chars().enumerate().for_each(|(index, character)| {
				let style = self.highlights.remove(&index);
				highlights.insert(index, HighlightedCharacter { character, style });
			});

			if let Some(extension) = &self.extension {
				extension.chars().enumerate().for_each(|(index, character)| {
					let next = self.stem.chars().count() + index;
					let style = self.highlights.remove(&next);
					highlights
						.insert(self.stem.chars().count() + index, HighlightedCharacter { character, style });
				});
			}

			let extension = highlights.split_off(self.stem.chars().count());
			let stem = highlights;
			(stem, extension)
		};

		let mut spans = spanify(stem_highlights);
		spans.extend(spanify(extension_highlights));

		spans
	}
}

fn spanify(highlights: Vec<HighlightedCharacter>) -> Vec<HighlightedSpan> {
	let mut spans = Vec::new();
	let mut current_index = 0;

	while let Some(this) = highlights.get(current_index) {
		let next_with_same_style =
			highlights[current_index + 1..].iter().take_while(|that| this.style == that.style);

		let content =
			this.character.to_string() + &next_with_same_style.map(|c| c.character).collect::<String>();

		current_index += content.len();
		let span = HighlightedSpan { content, style: this.style };
		spans.push(span);
	}

	spans
}

#[derive(Debug)]
pub struct HighlightedSpan {
	content: String,
	style:   Option<Style>,
}

impl From<HighlightedSpan> for yazi_plugin::elements::Span {
	fn from(value: HighlightedSpan) -> Self {
		yazi_plugin::elements::Span(ratatui::text::Span {
			content: value.content.into(),
			style:   value.style.map(Into::into).unwrap_or_default(),
		})
	}
}

#[derive(Debug)]
struct HighlightedCharacter {
	character: char,
	style:     Option<Style>,
}

#[cfg(test)]
mod tests {
	use yazi_shared::theme::StyleShadow;

	use super::*;

	fn create_style(fgcolor: ratatui::style::Color) -> yazi_shared::theme::Style {
		Style::from(StyleShadow { fg: Some(yazi_shared::theme::Color(fgcolor)), ..Default::default() })
	}

	#[test]
	fn test_highlight_stem() {
		let mut hl = HighlightedFilenameBuilder::new("filename".to_string(), None);
		// when there is no extension, the whole name is the stem

		let style = create_style(ratatui::style::Color::Blue);
		hl.add_highlight(0.."file".len(), style);
		let spans = hl.build_spans();
		assert_eq!(spans.len(), 2);

		assert_eq!(spans[0].content, "file");
		assert_eq!(spans[0].style, Some(style));

		assert_eq!(spans[1].content, "name");
		assert_eq!(spans[1].style, None);
	}

	#[test]
	fn test_highlight_extension() {
		let mut hl = HighlightedFilenameBuilder::new("filename".to_string(), Some(".txt".to_string()));

		let style = create_style(ratatui::style::Color::Blue);
		{
			// simulate the user highlighting the extension without the dot
			let stem_length = "filename.".chars().count();
			let full_length = stem_length + "txt".chars().count();
			hl.add_highlight(stem_length..full_length, style);
		}

		let spans = hl.build_spans();
		assert_eq!(spans.len(), 3);

		assert_eq!(spans[0].content, "filename");
		assert_eq!(spans[0].style, None);

		assert_eq!(spans[1].content, ".");
		assert_eq!(spans[1].style, None);

		assert_eq!(spans[2].content, "txt");
		assert_eq!(spans[2].style, Some(style));
	}

	#[test]
	fn test_highlight_nothing() {
		// verify that the stem and extension are separate spans even if they are
		// not highlighted
		let mut hl = HighlightedFilenameBuilder::new("notes".to_string(), Some(".txt".to_string()));
		let spans = hl.build_spans();

		assert_eq!(spans.len(), 2);

		assert_eq!(spans[0].content, "notes");
		assert_eq!(spans[0].style, None);

		assert_eq!(spans[1].content, ".txt");
		assert_eq!(spans[1].style, None);
	}

	#[test]
	fn test_empty_name() {
		// just to make sure it doesn't crash
		let mut hl = HighlightedFilenameBuilder::new("".to_string(), None);
		let spans = hl.build_spans();
		assert_eq!(spans.len(), 0);
	}

	#[test]
	fn test_highlight_across_extension_boundary() {
		// for "filename.txt", if the user highlights "name.t"
		let mut hl = HighlightedFilenameBuilder::new("filename".to_string(), Some(".txt".to_string()));
		let style = create_style(ratatui::style::Color::Blue);

		{
			let start = "file".chars().count();
			let end = "filename.t".chars().count();
			hl.add_highlight(start..end, style);
		}

		let spans = hl.build_spans();
		assert_eq!(spans.len(), 4);

		assert_eq!(spans[0].content, "file");
		assert_eq!(spans[0].style, None);

		assert_eq!(spans[1].content, "name");
		assert_eq!(spans[1].style, Some(style));

		assert_eq!(spans[2].content, ".t");
		assert_eq!(spans[2].style, Some(style));

		assert_eq!(spans[3].content, "xt");
		assert_eq!(spans[3].style, None);
	}

	#[test]
	fn test_add_extension_highlight() {
		let mut hl = HighlightedFilenameBuilder::new("filename".to_string(), Some(".txt".to_string()));
		let style = create_style(ratatui::style::Color::Blue);
		hl.add_extension_highlight(style);

		let spans = hl.build_spans();
		assert_eq!(spans.len(), 2);

		assert_eq!(spans[0].content, "filename");
		assert_eq!(spans[0].style, None);

		assert_eq!(spans[1].content, ".txt");
		assert_eq!(spans[1].style, Some(style));
	}

	#[test]
	fn test_multibyte_characters() {
		// multi-byte characters must not cause panics even if the file name is very
		// complex. Test both the stem and the extension.
		let mut hl = HighlightedFilenameBuilder::new(
			"pokémon-listing-🤔-スゴイ-H̐͌̃ē̐̅l̐͑̚l͋͋́o̐̒̓,̍̑̀ ̓̈́̚w͊̀̕o̓͆͝r̎͝͝l͑̑̇d̎̐̕! ".to_string(),
			Some(".🤔-スH̐͌̃ē̐̅l̐͑̚txt".to_string()),
		);
		let style = create_style(ratatui::style::Color::Blue);
		hl.add_highlight(0.."poké".chars().count(), style);
		hl.add_extension_highlight(style);

		let spans = hl.build_spans();

		assert_eq!(spans.len(), 3);
	}
}
