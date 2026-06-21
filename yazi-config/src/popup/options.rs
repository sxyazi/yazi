use ratatui_core::text::{Line, Text};
use ratatui_widgets::paragraph::{Paragraph, Wrap};
use yazi_binding::position::Position;
use yazi_macro::impl_data_any;
use yazi_shared::{strand::ToStrand, url::UrlBuf};

use crate::YAZI;

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

	pub fn trash(urls: &[yazi_shared::url::UrlBuf]) -> Self {
		Self::new(
			Self::replace_number(&YAZI.confirm.trash_title, urls.len()),
			YAZI.confirm.trash_position(),
			None,
			Self::truncate_list(urls, urls.len(), 100),
		)
	}

	pub fn delete(urls: &[yazi_shared::url::UrlBuf]) -> Self {
		Self::new(
			Self::replace_number(&YAZI.confirm.delete_title, urls.len()),
			YAZI.confirm.delete_position(),
			None,
			Self::truncate_list(urls, urls.len(), 100),
		)
	}

	pub fn overwrite(url: &UrlBuf) -> Self {
		Self::new(
			YAZI.confirm.overwrite_title.clone(),
			YAZI.confirm.overwrite_position(),
			Some(Text::raw(&YAZI.confirm.overwrite_body)),
			Some(url.to_strand().into_string_lossy().into()),
		)
	}

	pub fn quit(len: usize, names: Vec<String>) -> Self {
		Self::new(
			Self::replace_number(&YAZI.confirm.quit_title, len),
			YAZI.confirm.quit_position(),
			Some(Text::raw(&YAZI.confirm.quit_body)),
			Self::truncate_list(names, len, 10),
		)
	}

	fn replace_number(tpl: &str, n: usize) -> String {
		tpl.replace("{n}", &n.to_string()).replace("{s}", if n > 1 { "s" } else { "" })
	}

	fn truncate_list<I>(it: I, len: usize, max: usize) -> Option<Text<'static>>
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
}
