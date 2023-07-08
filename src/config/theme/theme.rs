use std::{fs, path::PathBuf};

use serde::Deserialize;
use xdg::BaseDirectories;

use super::{Color, Icon};
use crate::misc::absolute_path;

#[derive(Deserialize)]
pub struct Mode {
	pub normal:   Color,
	pub select:   Color,
	pub unselect: Color,
}

#[derive(Deserialize)]
pub struct Tab {
	pub active:   Color,
	pub inactive: Color,
}

#[derive(Deserialize)]
pub struct Selection {
	pub normal:   Color,
	pub hovered:  Color,
	pub selected: Color,
}

#[derive(Deserialize)]
pub struct Filetype {}

#[derive(Deserialize)]
pub struct Syntect {
	pub theme: PathBuf,
}

#[derive(Deserialize)]
pub struct Theme {
	pub mode:      Mode,
	pub tab:       Tab,
	pub selection: Selection,
	pub filetype:  Filetype,
	pub syntect:   Syntect,
	#[serde(deserialize_with = "Icon::deserialize")]
	pub icons:     Vec<Icon>,
}

impl Theme {
	pub fn new() -> Self {
		let path = BaseDirectories::new().unwrap().get_config_file("yazi/theme.toml");

		let mut parsed: Self = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
		parsed.syntect.theme = absolute_path(&parsed.syntect.theme);
		parsed
	}
}
