use super::{Offset, Position};
use crate::{INPUT, SELECT};

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

impl InputCfg {
	#[inline]
	pub fn cd() -> Self {
		Self {
			title: INPUT.cd_title.to_owned(),
			position: Position::new(INPUT.cd_origin, INPUT.cd_offset),
			completion: true,
			..Default::default()
		}
	}

	#[inline]
	pub fn create() -> Self {
		Self {
			title: INPUT.create_title.to_owned(),
			position: Position::new(INPUT.create_origin, INPUT.create_offset),
			..Default::default()
		}
	}

	#[inline]
	pub fn rename() -> Self {
		Self {
			title: INPUT.rename_title.to_owned(),
			position: Position::new(INPUT.rename_origin, INPUT.rename_offset),
			..Default::default()
		}
	}

	#[inline]
	pub fn trash(n: usize) -> Self {
		let title = INPUT.trash_title.replace("{n}", &n.to_string());
		Self {
			title: title.replace("{s}", if n > 1 { "s" } else { "" }),
			position: Position::new(INPUT.trash_origin, INPUT.trash_offset),
			..Default::default()
		}
	}

	#[inline]
	pub fn delete(n: usize) -> Self {
		let title = INPUT.delete_title.replace("{n}", &n.to_string());
		Self {
			title: title.replace("{s}", if n > 1 { "s" } else { "" }),
			position: Position::new(INPUT.delete_origin, INPUT.delete_offset),
			..Default::default()
		}
	}

	#[inline]
	pub fn filter() -> Self {
		Self {
			title: INPUT.filter_title.to_owned(),
			position: Position::new(INPUT.filter_origin, INPUT.filter_offset),
			realtime: true,
			..Default::default()
		}
	}

	#[inline]
	pub fn find(prev: bool) -> Self {
		Self {
			title: INPUT.find_title[prev as usize].to_owned(),
			position: Position::new(INPUT.find_origin, INPUT.find_offset),
			realtime: true,
			..Default::default()
		}
	}

	#[inline]
	pub fn search(name: &str) -> Self {
		Self {
			title: INPUT.search_title.replace("{n}", name),
			position: Position::new(INPUT.search_origin, INPUT.search_offset),
			..Default::default()
		}
	}

	#[inline]
	pub fn shell(block: bool) -> Self {
		Self {
			title: INPUT.shell_title[block as usize].to_owned(),
			position: Position::new(INPUT.shell_origin, INPUT.shell_offset),
			highlight: true,
			..Default::default()
		}
	}

	#[inline]
	pub fn overwrite() -> Self {
		Self {
			title: INPUT.overwrite_title.to_owned(),
			position: Position::new(INPUT.overwrite_origin, INPUT.overwrite_offset),
			..Default::default()
		}
	}

	#[inline]
	pub fn quit(n: usize) -> Self {
		let title = INPUT.quit_title.replace("{n}", &n.to_string());
		Self {
			title: title.replace("{s}", if n > 1 { "s" } else { "" }),
			position: Position::new(INPUT.quit_origin, INPUT.quit_offset),
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

impl SelectCfg {
	#[inline]
	fn max_height(len: usize) -> u16 {
		SELECT.open_offset.height.min(SELECT.border().saturating_add(len as u16))
	}

	#[inline]
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
