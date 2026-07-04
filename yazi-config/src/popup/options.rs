use std::slice;

use ratatui_core::text::{Line, Span, Text};
use ratatui_widgets::paragraph::{Paragraph, Wrap};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use yazi_binding::position::Position;
use yazi_fs::file::File;
use yazi_macro::impl_data_any;
use yazi_shared::strand::ToStrand;

use crate::{THEME, YAZI};

// Mirrors the horizontal `Margin::new(2, 0)` the confirm list widget applies
// around its content (yazi-fm/src/confirm/list.rs), so a path truncated here
// to the popup's configured width doesn't still overflow once rendered.
const LIST_MARGIN: u16 = 4;

// --- PickCfg
#[derive(Clone, Debug, Default)]
pub struct PickCfg {
	pub title:    String,
	pub items:    Vec<String>,
	pub position: Position,
}

impl_data_any!(PickCfg);

// --- ConfirmCfg
#[derive(Clone, Debug, Default)]
pub struct ConfirmCfg {
	pub position: Position,
	pub title:    Line<'static>,
	pub body:     Paragraph<'static>,
	pub list:     Paragraph<'static>,
}

impl_data_any!(ConfirmCfg);

impl ConfirmCfg {
	fn new(
		title: String,
		position: Position,
		body: Option<Text<'static>>,
		list: Option<Text<'static>>,
	) -> Self {
		Self {
			position,
			title: Line::raw(title),
			body: body.map(|b| Paragraph::new(b).wrap(Wrap { trim: false })).unwrap_or_default(),
			list: list.map(|l| Paragraph::new(l).wrap(Wrap { trim: false })).unwrap_or_default(),
		}
	}

	pub fn trash(files: &[File]) -> Self {
		let position = YAZI.confirm.trash_position();
		Self::new(
			Self::replace_number(&YAZI.confirm.trash_title, files.len()),
			position,
			None,
			Self::truncate_files(files, 100, position.width),
		)
	}

	pub fn delete(files: &[File]) -> Self {
		let position = YAZI.confirm.delete_position();
		Self::new(
			Self::replace_number(&YAZI.confirm.delete_title, files.len()),
			position,
			None,
			Self::truncate_files(files, 100, position.width),
		)
	}

	pub fn overwrite(file: &File) -> Self {
		let position = YAZI.confirm.overwrite_position();
		Self::new(
			YAZI.confirm.overwrite_title.clone(),
			position,
			Some(Text::raw(&YAZI.confirm.overwrite_body)),
			Self::truncate_files(slice::from_ref(file), 1, position.width),
		)
	}

	pub fn quit(len: usize, names: Vec<String>) -> Self {
		Self::new(
			Self::replace_number(&YAZI.confirm.quit_title, len),
			YAZI.confirm.quit_position(),
			Some(Text::raw(&YAZI.confirm.quit_body)),
			Self::truncate_lines(names, len, 10),
		)
	}

	fn replace_number(tpl: &str, n: usize) -> String {
		tpl.replace("{n}", &n.to_string()).replace("{s}", if n > 1 { "s" } else { "" })
	}

	fn truncate_lines<I>(it: I, len: usize, max: usize) -> Option<Text<'static>>
	where
		I: IntoIterator,
		I::Item: ToStrand,
	{
		let mut lines = Vec::with_capacity(len.min(max + 1));
		for (i, s) in it.into_iter().enumerate() {
			if i >= max {
				lines.push(format!("... and {} more", len - max));
				break;
			}
			lines.push(s.to_strand().into_string_lossy());
		}
		Some(Text::from_iter(lines))
	}

	fn truncate_files(files: &[File], max: usize, width: u16) -> Option<Text<'static>> {
		let budget = (width as usize).saturating_sub(LIST_MARGIN as usize);

		let mut lines = Vec::with_capacity(files.len().min(max + 1));
		for (i, f) in files.iter().enumerate() {
			if i >= max {
				lines.push(Line::raw(format!("... and {} more", files.len() - max)));
				break;
			}

			lines.push(Line::default());
			let mut prefix = 0;
			if YAZI.confirm.show_icons
				&& let Some(icon) = THEME.icon.matches(f, false)
			{
				prefix = icon.text.width() + 1;
				lines[i].push_span(Span::styled(icon.text, icon.style));
				lines[i].push_span(" ");
			}

			let path = f.url.to_strand().into_string_lossy();
			let path = if YAZI.confirm.truncate_paths {
				Self::truncate_rtl(&path, budget.saturating_sub(prefix))
			} else {
				path
			};
			lines[i].push_span(path);
		}
		Some(lines.into())
	}

	// Truncates `s` from the left with a leading `…`, keeping its tail, since
	// the end of a path is usually more identifying than its start.
	fn truncate_rtl(s: &str, max: usize) -> String {
		if s.width() <= max {
			return s.to_owned();
		}
		if max == 0 {
			return String::new();
		}

		let mut adv = 0;
		let mut idx = s.len();
		for (i, c) in s.char_indices().rev() {
			let w = c.width().unwrap_or(0);
			if adv + w > max.saturating_sub(1) {
				break;
			}
			adv += w;
			idx = i;
		}

		format!("…{}", &s[idx..])
	}
}
