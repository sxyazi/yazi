use ratatui::{text::{Line, Text}, widgets::{Paragraph, Wrap}};
use yazi_shared::url::Url;

use super::{Offset, Origin, Position};
use crate::YAZI;

#[derive(Default)]
pub struct InputCfg {
	pub title:      String,
	pub value:      String,
	pub cursor:     Option<usize>,
	pub obscure:    bool,
	pub position:   Position,
	pub realtime:   bool,
	pub completion: bool,
}

#[derive(Default)]
pub struct PickCfg {
	pub title:    String,
	pub items:    Vec<String>,
	pub position: Position,
}

#[derive(Default)]
pub struct ConfirmCfg {
	pub position: Position,
	pub title:    Line<'static>,
	pub body:     Paragraph<'static>,
	pub list:     Paragraph<'static>,
}

impl InputCfg {
	pub fn cd() -> Self {
		Self {
			title: YAZI.input.cd_title.to_owned(),
			position: Position::new(YAZI.input.cd_origin, YAZI.input.cd_offset),
			completion: true,
			..Default::default()
		}
	}

	pub fn create(dir: bool) -> Self {
		Self {
			title: YAZI.input.create_title[dir as usize].to_owned(),
			position: Position::new(YAZI.input.create_origin, YAZI.input.create_offset),
			..Default::default()
		}
	}

	pub fn rename() -> Self {
		Self {
			title: YAZI.input.rename_title.to_owned(),
			position: Position::new(YAZI.input.rename_origin, YAZI.input.rename_offset),
			..Default::default()
		}
	}

	pub fn filter() -> Self {
		Self {
			title: YAZI.input.filter_title.to_owned(),
			position: Position::new(YAZI.input.filter_origin, YAZI.input.filter_offset),
			realtime: true,
			..Default::default()
		}
	}

	pub fn find(prev: bool) -> Self {
		Self {
			title: YAZI.input.find_title[prev as usize].to_owned(),
			position: Position::new(YAZI.input.find_origin, YAZI.input.find_offset),
			realtime: true,
			..Default::default()
		}
	}

	pub fn search(name: &str) -> Self {
		Self {
			title: YAZI.input.search_title.replace("{n}", name),
			position: Position::new(YAZI.input.search_origin, YAZI.input.search_offset),
			..Default::default()
		}
	}

	pub fn shell(block: bool) -> Self {
		Self {
			title: YAZI.input.shell_title[block as usize].to_owned(),
			position: Position::new(YAZI.input.shell_origin, YAZI.input.shell_offset),
			..Default::default()
		}
	}

	#[inline]
	pub fn with_value(mut self, value: impl Into<String>) -> Self {
		self.value = value.into();
		self
	}

	#[inline]
	pub fn with_cursor(mut self, cursor: Option<usize>) -> Self {
		self.cursor = cursor;
		self
	}
}

impl ConfirmCfg {
	fn new(
		title: String,
		(origin, offset): (Origin, Offset),
		body: Option<Text<'static>>,
		list: Option<Text<'static>>,
	) -> Self {
		Self {
			position: Position::new(origin, offset),
			title:    Line::raw(title),
			body:     body.map(|c| Paragraph::new(c).wrap(Wrap { trim: false })).unwrap_or_default(),
			list:     list.map(|l| Paragraph::new(l).wrap(Wrap { trim: false })).unwrap_or_default(),
		}
	}

	pub fn trash(urls: &[yazi_shared::url::Url]) -> Self {
		Self::new(
			Self::replace_number(&YAZI.confirm.trash_title, urls.len()),
			(YAZI.confirm.trash_origin, YAZI.confirm.trash_offset),
			None,
			Self::truncate_list(urls.iter(), urls.len(), 100),
		)
	}

	pub fn delete(urls: &[yazi_shared::url::Url]) -> Self {
		Self::new(
			Self::replace_number(&YAZI.confirm.delete_title, urls.len()),
			(YAZI.confirm.delete_origin, YAZI.confirm.delete_offset),
			None,
			Self::truncate_list(urls.iter(), urls.len(), 100),
		)
	}

	pub fn overwrite(url: &Url) -> Self {
		Self::new(
			YAZI.confirm.overwrite_title.to_owned(),
			(YAZI.confirm.overwrite_origin, YAZI.confirm.overwrite_offset),
			Some(Text::raw(&YAZI.confirm.overwrite_body)),
			Some(url.to_string().into()),
		)
	}

	pub fn quit(len: usize, names: Vec<String>) -> Self {
		Self::new(
			Self::replace_number(&YAZI.confirm.quit_title, len),
			(YAZI.confirm.quit_origin, YAZI.confirm.quit_offset),
			Some(Text::raw(&YAZI.confirm.quit_body)),
			Self::truncate_list(names.into_iter(), len, 10),
		)
	}

	fn replace_number(tpl: &str, n: usize) -> String {
		tpl.replace("{n}", &n.to_string()).replace("{s}", if n > 1 { "s" } else { "" })
	}

	fn truncate_list(
		it: impl Iterator<Item = impl Into<String>>,
		len: usize,
		max: usize,
	) -> Option<Text<'static>> {
		let mut lines = Vec::with_capacity(len.min(max + 1));
		for (i, s) in it.enumerate() {
			if i >= max {
				lines.push(format!("... and {} more", len - max));
				break;
			}
			lines.push(s.into());
		}
		Some(Text::from_iter(lines))
	}
}

impl PickCfg {
	#[inline]
	fn max_height(len: usize) -> u16 {
		YAZI.pick.open_offset.height.min(YAZI.pick.border().saturating_add(len as u16))
	}

	pub fn open(items: Vec<String>) -> Self {
		let max_height = Self::max_height(items.len());
		Self {
			title: YAZI.pick.open_title.to_owned(),
			items,
			position: Position::new(YAZI.pick.open_origin, Offset {
				height: max_height,
				..YAZI.pick.open_offset
			}),
		}
	}
}
