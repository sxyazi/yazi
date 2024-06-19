use yazi_shared::fs::Url;

use super::{Offset, Position};
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
	pub content:  String,
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
	#[inline]
	pub fn delete(targets: &[yazi_shared::fs::Url]) -> Self {
		Self {
			title:    CONFIRM.delete_title.replace("{n}", &targets.len().to_string()),
			position: Position::new(CONFIRM.delete_origin, CONFIRM.delete_offset),
			content:  targets.iter().map(|t| t.to_string()).collect::<Vec<_>>().join("\n"),
		}
	}

	#[inline]
	pub fn trash(targets: &[yazi_shared::fs::Url]) -> Self {
		Self {
			title:    CONFIRM.trash_title.replace("{n}", &targets.len().to_string()),
			position: Position::new(CONFIRM.trash_origin, CONFIRM.trash_offset),
			content:  targets.iter().map(|t| t.to_string()).collect::<Vec<_>>().join("\n"),
		}
	}

	#[inline]
	pub fn overwrite(url: &Url) -> Self {
		Self {
			title:    CONFIRM.overwrite_title.to_owned(),
			content:  CONFIRM.overwrite_content.replace("{url}", &url.to_string()),
			position: Position::new(CONFIRM.overwrite_origin, CONFIRM.overwrite_offset),
		}
	}

	#[inline]
	pub fn quit(ongoing_task_names: Vec<String>) -> Self {
		let n = ongoing_task_names.len();
		let mut message = CONFIRM.quit_content.replace("{n}", &n.to_string());

		message.push_str(&ongoing_task_names.join("\n"));

		Self {
			title:    CONFIRM.quit_title.to_owned(),
			content:  message,
			position: Position::new(CONFIRM.quit_origin, CONFIRM.quit_offset),
		}
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
