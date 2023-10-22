use anyhow::{bail, Result};
use syntect::{easy::HighlightLines, util::as_24_bit_terminal_escaped};

use super::Input;
use crate::highlighter;

impl Input {
	pub fn value_pretty(&self) -> Result<String> {
		if !self.highlight {
			bail!("Highlighting is disabled")
		}

		let (syntaxes, theme) = highlighter();
		if let Some(syntax) = syntaxes.find_syntax_by_name("Bourne Again Shell (bash)") {
			let mut h = HighlightLines::new(syntax, theme);
			let regions = h.highlight_line(self.value(), syntaxes)?;
			return Ok(as_24_bit_terminal_escaped(&regions, false));
		}

		bail!("Failed to find syntax")
	}
}
