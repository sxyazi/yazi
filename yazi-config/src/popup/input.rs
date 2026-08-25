use serde::Deserialize;
use yazi_binding::position::{Offset, Origin, Position};
use yazi_codegen::{DeserializeOver, DeserializeOver2};
use yazi_shared::{spec::EncodeSpec, url::Url};
use yazi_widgets::input::InputOpt;

#[derive(Deserialize, DeserializeOver, DeserializeOver2)]
pub struct Input {
	pub cursor_blink: bool,

	// cd
	cd_title:  String,
	cd_origin: Origin,
	cd_offset: Offset,

	// create
	create_title:  [String; 2],
	create_origin: Origin,
	create_offset: Offset,

	// rename
	rename_title:  String,
	rename_origin: Origin,
	rename_offset: Offset,

	// filter
	filter_title:  String,
	filter_origin: Origin,
	filter_offset: Offset,

	// find
	find_title:  [String; 2],
	find_origin: Origin,
	find_offset: Offset,

	// search
	search_title:  String,
	search_origin: Origin,
	search_offset: Offset,

	// shell
	shell_title:  [String; 2],
	shell_origin: Origin,
	shell_offset: Offset,
}

impl Input {
	pub fn cd(&self, cwd: Url) -> InputOpt {
		InputOpt {
			name: "cd".to_owned(),
			title: self.cd_title.clone(),
			value: if cwd.kind().is_local() { String::new() } else { EncodeSpec(cwd).to_string() },
			history: "shared".to_owned(),
			position: Position::new(self.cd_origin, self.cd_offset),
			completion: true,
			..Default::default()
		}
	}

	pub fn create(&self, dir: bool) -> InputOpt {
		InputOpt {
			name: format!("create-{}", if dir { "dir" } else { "file" }),
			title: self.create_title[dir as usize].clone(),
			history: "shared".to_owned(),
			position: Position::new(self.create_origin, self.create_offset),
			..Default::default()
		}
	}

	pub fn rename(&self, is_dir: bool) -> InputOpt {
		InputOpt {
			name: format!("rename-{}", if is_dir { "dir" } else { "file" }),
			title: self.rename_title.clone(),
			history: "shared".to_owned(),
			position: Position::new(self.rename_origin, self.rename_offset),
			..Default::default()
		}
	}

	pub fn filter(&self) -> InputOpt {
		InputOpt {
			name: "filter".to_owned(),
			title: self.filter_title.clone(),
			history: "shared".to_owned(),
			position: Position::new(self.filter_origin, self.filter_offset),
			realtime: true,
			..Default::default()
		}
	}

	pub fn find(&self, prev: bool) -> InputOpt {
		InputOpt {
			name: "find".to_owned(),
			title: self.find_title[prev as usize].clone(),
			history: "shared".to_owned(),
			position: Position::new(self.find_origin, self.find_offset),
			realtime: true,
			..Default::default()
		}
	}

	pub fn search(&self, name: &str) -> InputOpt {
		InputOpt {
			name: "search".to_owned(),
			title: self.search_title.replace("{n}", name),
			history: "shared".to_owned(),
			position: Position::new(self.search_origin, self.search_offset),
			..Default::default()
		}
	}

	pub fn shell(&self, block: bool) -> InputOpt {
		InputOpt {
			name: "shell".to_owned(),
			title: self.shell_title[block as usize].clone(),
			history: "shared".to_owned(),
			position: Position::new(self.shell_origin, self.shell_offset),
			..Default::default()
		}
	}

	pub fn tab_rename(&self) -> InputOpt {
		InputOpt {
			name: "tab-rename".to_owned(),
			title: "Rename tab:".to_owned(),
			history: "shared".to_owned(),
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
