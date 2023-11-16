use super::Position;
use crate::INPUT;

#[derive(Default)]
pub struct InputOpt {
	pub title:      String,
	pub value:      String,
	pub position:   Position,
	pub realtime:   bool,
	pub completion: bool,
	pub highlight:  bool,
}

#[derive(Default)]
pub struct SelectOpt {
	pub title:    String,
	pub items:    Vec<String>,
	pub position: Position,
}

impl InputOpt {
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
	pub fn find(prev: bool) -> Self {
		Self {
			title: INPUT.find_title[prev as usize].to_owned(),
			position: Position::new(INPUT.find_origin, INPUT.find_offset),
			realtime: true,
			..Default::default()
		}
	}

	#[inline]
	pub fn search() -> Self {
		Self {
			title: INPUT.search_title.to_owned(),
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
	pub fn with_value(mut self, value: impl Into<String>) -> Self {
		self.value = value.into();
		self
	}
}

impl SelectOpt {
	#[inline]
	pub fn open() -> Self { todo!() }

	#[inline]
	pub fn with_items(mut self, items: Vec<String>) -> Self {
		self.items = items;
		self
	}
}
