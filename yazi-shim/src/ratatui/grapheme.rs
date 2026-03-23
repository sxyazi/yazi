// Copied from https://github.com/ratatui/ratatui/blob/main/ratatui-core/src/text/grapheme.rs
use std::borrow::Cow;

use ratatui::style::{Style, Styled};

const NBSP: &str = "\u{00a0}";
const ZWSP: &str = "\u{200b}";

/// A grapheme associated to a style.
/// Note that, although `StyledGrapheme` is the smallest divisible unit of text,
/// it actually is not a member of the text type hierarchy (`Text` -> `Line` ->
/// `Span`). It is a separate type used mostly for rendering purposes. A `Span`
/// consists of components that can be split into `StyledGrapheme`s, but it does
/// not contain a collection of `StyledGrapheme`s.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct StyledGrapheme<'a> {
	pub symbol: Cow<'a, str>,
	pub style:  Style,
}

impl<'a> StyledGrapheme<'a> {
	/// Creates a new `StyledGrapheme` with the given symbol and style.
	///
	/// `style` accepts any type that is convertible to [`Style`] (e.g. [`Style`],
	/// [`Color`], or your own type that implements [`Into<Style>`]).
	///
	/// [`Color`]: crate::style::Color
	pub fn new<S: Into<Cow<'a, str>>>(symbol: S, style: Style) -> Self {
		Self { symbol: symbol.into(), style }
	}

	pub fn is_whitespace(&self) -> bool {
		let symbol = &*self.symbol;
		symbol == ZWSP || symbol.chars().all(char::is_whitespace) && symbol != NBSP
	}
}

impl Styled for StyledGrapheme<'_> {
	type Item = Self;

	fn style(&self) -> Style { self.style }

	fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
		self.style = style.into();
		self
	}
}
