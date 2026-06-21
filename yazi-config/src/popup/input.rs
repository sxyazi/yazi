use serde::Deserialize;
use yazi_binding::position::{Offset, Origin, Position};
use yazi_codegen::{DeserializeOver, DeserializeOver2};
use yazi_shared::{scheme::Encode as EncodeScheme, url::Url};
use yazi_widgets::input::InputOpt;

#[derive(Deserialize, DeserializeOver, DeserializeOver2)]
pub struct Input {
	pub cursor_blink: bool,

	// cd
	pub cd_title:  String,
	pub cd_origin: Origin,
	pub cd_offset: Offset,

	// create
	pub create_title:  [String; 2],
	pub create_origin: Origin,
	pub create_offset: Offset,

	// rename
	pub rename_title:  String,
	pub rename_origin: Origin,
	pub rename_offset: Offset,

	// filter
	pub filter_title:  String,
	pub filter_origin: Origin,
	pub filter_offset: Offset,

	// find
	pub find_title:  [String; 2],
	pub find_origin: Origin,
	pub find_offset: Offset,

	// search
	pub search_title:  String,
	pub search_origin: Origin,
	pub search_offset: Offset,

	// shell
	pub shell_title:  [String; 2],
	pub shell_origin: Origin,
	pub shell_offset: Offset,
}

impl Input {
	pub fn cd(&self, cwd: Url) -> InputOpt {
		InputOpt {
			title: self.cd_title.clone(),
			value: if cwd.kind().is_local() { String::new() } else { EncodeScheme(cwd).to_string() },
			position: Position::new(self.cd_origin, self.cd_offset),
			completion: true,
			..Default::default()
		}
	}

	pub fn create(&self, dir: bool) -> InputOpt {
		InputOpt {
			title: self.create_title[dir as usize].clone(),
			position: Position::new(self.create_origin, self.create_offset),
			..Default::default()
		}
	}

	pub fn rename(&self) -> InputOpt {
		InputOpt {
			title: self.rename_title.clone(),
			position: Position::new(self.rename_origin, self.rename_offset),
			..Default::default()
		}
	}

	pub fn filter(&self) -> InputOpt {
		InputOpt {
			title: self.filter_title.clone(),
			position: Position::new(self.filter_origin, self.filter_offset),
			realtime: true,
			..Default::default()
		}
	}

	pub fn find(&self, prev: bool) -> InputOpt {
		InputOpt {
			title: self.find_title[prev as usize].clone(),
			position: Position::new(self.find_origin, self.find_offset),
			realtime: true,
			..Default::default()
		}
	}

	pub fn search(&self, name: &str) -> InputOpt {
		InputOpt {
			title: self.search_title.replace("{n}", name),
			position: Position::new(self.search_origin, self.search_offset),
			..Default::default()
		}
	}

	pub fn shell(&self, block: bool) -> InputOpt {
		InputOpt {
			title: self.shell_title[block as usize].clone(),
			position: Position::new(self.shell_origin, self.shell_offset),
			..Default::default()
		}
	}

	pub fn tab_rename(&self) -> InputOpt {
		InputOpt {
			title: "Rename tab:".to_owned(),
			position: Position::new(Origin::TopCenter, Offset {
				x:      0,
				y:      2,
				width:  50,
				height: 3,
			}),
			..Default::default()
		}
	}
}
