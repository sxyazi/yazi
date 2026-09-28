use ratatui_core::{buffer::Buffer, layout::Rect, text::{Line, Span}, widgets::Widget};
use yazi_config::{THEME, keymap::{Chord, ChordArc, Key}};

pub(super) enum Cand<'a> {
	/// A chord, shown with the keys it still needs.
	Chord { chord: &'a Chord, times: usize },
	/// The chords that need more than one key after `key`, folded under it.
	Group { key: &'a Key, label: Option<&'a Chord>, members: Vec<&'a Chord> },
}

impl<'a> Cand<'a> {
	/// The rows for the candidates, in their order. With `fold`, a chord that needs one more
	/// key stands alone, and those that need more are folded by their next key, under the label
	/// that a chord without `run` on the same keys gives the group. Without it, every chord that
	/// can run is a row of its own.
	pub(super) fn rows(cands: &'a [ChordArc], times: usize, fold: bool) -> Vec<Self> {
		if !fold {
			return cands
				.iter()
				.filter(|c| !c.is_label())
				.map(|c| Self::Chord { chord: c, times })
				.collect();
		}

		let mut rows: Vec<Self> = Vec::with_capacity(cands.len());
		for chord in cands.iter().map(|c| &**c) {
			// The label of the group whose keys are already typed names no row
			let Some(key) = chord.on.get(times) else { continue };
			let single = chord.on.len() == times + 1;
			if single && !chord.is_label() {
				rows.push(Self::Chord { chord, times });
				continue;
			}

			let group = rows.iter_mut().find_map(|row| match row {
				Self::Group { key: k, label, members } if *k == key => Some((label, members)),
				_ => None,
			});
			match (group, single) {
				(Some((label, _)), true) => *label = Some(chord),
				(Some((_, members)), false) => members.push(chord),
				(None, true) => rows.push(Self::Group { key, label: Some(chord), members: vec![] }),
				(None, false) => rows.push(Self::Group { key, label: None, members: vec![chord] }),
			}
		}

		// A label with nothing under it has nothing to show, and a single unlabelled chord is
		// clearer as itself than as a group of one
		rows
			.into_iter()
			.filter_map(|row| match row {
				Self::Group { members, .. } if members.is_empty() => None,
				Self::Group { label: None, members, .. } if members.len() == 1 => {
					Some(Self::Chord { chord: members[0], times })
				}
				row => Some(row),
			})
			.collect()
	}

	fn keys(&self) -> Vec<String> {
		match self {
			Self::Chord { chord, times } => chord.on[*times..].iter().map(ToString::to_string).collect(),
			Self::Group { key, .. } => vec![key.to_string()],
		}
	}

	fn desc(&self) -> String {
		match self {
			Self::Chord { chord, .. } => chord.desc_or_run().into_owned(),
			Self::Group { label: Some(label), .. } if !label.desc.is_empty() => {
				format!("+{}", label.desc_or_run())
			}
			Self::Group { members, .. } => format!("+{}", members.len()),
		}
	}
}

impl Widget for Cand<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let keys = self.keys();
		let mut spans = Vec::with_capacity(10);
		let separator = &**THEME.which.separator.load();

		// Padding
		spans.push(Span::raw(" ".repeat(10usize.saturating_sub(keys.join("").len()))));

		// First key
		spans.push(Span::styled(keys[0].clone(), THEME.which.cand.get()));

		// Rest keys
		spans.extend(keys.iter().skip(1).map(|k| Span::styled(k.clone(), THEME.which.rest.get())));

		// Separator
		spans.push(Span::styled(separator, THEME.which.separator_style.get()));

		// Description
		spans.push(Span::styled(self.desc(), THEME.which.desc.get()));

		Line::from(spans).render(area, buf);
	}
}
