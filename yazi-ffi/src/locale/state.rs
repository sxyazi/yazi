use super::Width;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub(super) enum State {
	/// Outside quotes or `%` directives: `y` in `yyyy-MM-dd`, or `-` in `%Y-%m-%d`.
	Pattern,
	/// Inside quotes: `de` in `d 'de' MMMM 'de' y` is text, not date fields.
	Quoted,
	/// Just read `%`: `%Y` waits for `Y`; `%%` produces a literal percent sign.
	Percent,
	/// Skipping modifiers: `%_5EY` records `_` as no zero-padding and skips `5` and `E`.
	Modified(Width),
}
