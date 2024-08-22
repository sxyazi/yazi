use ratatui::{text::Text, widgets::{Paragraph, Wrap}};
use yazi_shared::fs::Url;

use super::{Offset, Origin, Position};
use crate::{CONFIRM, INPUT, SELECT};

#[derive(Default)]
pub struct InputCfg {
	pub title:      String,
	pub value:      String,
	pub cursor:     Option<usize>,
	pub position:   Position,
	pub realtime:   bool,
	pub completion: bool,
	pub highlight:  bool,
}

#[derive(Default)]
pub struct SelectCfg {
	pub title:    String,
	pub items:    Vec<String>,
	pub position: Position,
}

#[derive(Default)]
pub struct ConfirmCfg {
	pub title:    String,
	pub position: Position,
	pub content:  Paragraph<'static>,
	pub list:     Paragraph<'static>,
}

impl InputCfg {
	pub fn cd() -> Self {
		Self {
			title: INPUT.cd_title.to_owned(),
			position: Position::new(INPUT.cd_origin, INPUT.cd_offset),
			completion: true,
			..Default::default()
		}
	}

	pub fn create() -> Self {
		Self {
			title: INPUT.create_title.to_owned(),
			position: Position::new(INPUT.create_origin, INPUT.create_offset),
			..Default::default()
		}
	}

	pub fn rename() -> Self {
		Self {
			title: INPUT.rename_title.to_owned(),
			position: Position::new(INPUT.rename_origin, INPUT.rename_offset),
			..Default::default()
		}
	}

	pub fn filter() -> Self {
		Self {
			title: INPUT.filter_title.to_owned(),
			position: Position::new(INPUT.filter_origin, INPUT.filter_offset),
			realtime: true,
			..Default::default()
		}
	}

	pub fn find(prev: bool) -> Self {
		Self {
			title: INPUT.find_title[prev as usize].to_owned(),
			position: Position::new(INPUT.find_origin, INPUT.find_offset),
			realtime: true,
			..Default::default()
		}
	}

	pub fn search(name: &str) -> Self {
		Self {
			title: INPUT.search_title.replace("{n}", name),
			position: Position::new(INPUT.search_origin, INPUT.search_offset),
			..Default::default()
		}
	}

	pub fn shell(block: bool) -> Self {
		Self {
			title: INPUT.shell_title[block as usize].to_owned(),
			position: Position::new(INPUT.shell_origin, INPUT.shell_offset),
			highlight: true,
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
		content: Option<Text<'static>>,
		list: Option<Text<'static>>,
	) -> Self {
		Self {
			title,
			position: Position::new(origin, offset),
			content: content.map(|c| Paragraph::new(c).wrap(Wrap { trim: false })).unwrap_or_default(),
			list: list.map(|l| Paragraph::new(l).wrap(Wrap { trim: false })).unwrap_or_default(),
		}
	}

	pub fn trash(urls: &[yazi_shared::fs::Url]) -> Self {
		Self::new(
			Self::replace_number(&CONFIRM.trash_title, urls.len(), usize::MAX),
			(CONFIRM.trash_origin, CONFIRM.trash_offset),
			None,
			Some(urls.iter().map(ToString::to_string).collect()),
		)
	}

	pub fn delete(urls: &[yazi_shared::fs::Url]) -> Self {
		Self::new(
			Self::replace_number(&CONFIRM.delete_title, urls.len(), usize::MAX),
			(CONFIRM.delete_origin, CONFIRM.delete_offset),
			None,
			Some(urls.iter().map(ToString::to_string).collect()),
		)
	}

	pub fn overwrite(url: &Url) -> Self {
		Self::new(
			CONFIRM.overwrite_title.to_owned(),
			(CONFIRM.overwrite_origin, CONFIRM.overwrite_offset),
			Some(Text::raw(&CONFIRM.overwrite_content)),
			Some(url.to_string().into()),
		)
	}

	pub fn quit(left: Vec<String>) -> Self {
		Self::new(
			Self::replace_number(&CONFIRM.quit_title, left.len(), 10),
			(CONFIRM.quit_origin, CONFIRM.quit_offset),
			Some(Text::raw(&CONFIRM.quit_content)),
			Some(left.into_iter().collect()),
		)
	}

	fn replace_number(tpl: &str, n: usize, max: usize) -> String {
		let s = tpl.replace("{s}", if n > 1 { "s" } else { "" });
		s.replace("{n}", &if n > max { format!("{max}+") } else { n.to_string() })
	}
}

impl SelectCfg {
	#[inline]
	fn max_height(len: usize) -> u16 {
		SELECT.open_offset.height.min(SELECT.border().saturating_add(len as u16))
	}

	pub fn open(items: Vec<String>) -> Self {
		let max_height = Self::max_height(items.len());
		Self {
			title: SELECT.open_title.to_owned(),
			items,
			position: Position::new(SELECT.open_origin, Offset {
				height: max_height,
				..SELECT.open_offset
			}),
		}
	}
}
