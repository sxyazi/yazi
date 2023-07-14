use std::{fs, path::PathBuf};

use serde::Deserialize;
use xdg::BaseDirectories;

use super::{ColorDual, Filetype, Icon};
use crate::misc::absolute_path;

#[derive(Deserialize)]
pub struct Mode {
	pub normal:   ColorDual,
	pub select:   ColorDual,
	pub unselect: ColorDual,
}

#[derive(Deserialize)]
pub struct Tab {
	pub active:   ColorDual,
	pub inactive: ColorDual,
}

#[derive(Deserialize)]
pub struct Selection {
	pub normal:   ColorDual,
	pub hovered:  ColorDual,
	pub selected: ColorDual,
}

#[derive(Deserialize)]
pub struct Syntect {
	pub theme: PathBuf,
}

#[derive(Deserialize)]
pub struct Theme {
	pub mode:      Mode,
	pub tab:       Tab,
	pub selection: Selection,
	#[serde(deserialize_with = "Filetype::deserialize")]
	pub filetypes: Vec<Filetype>,
	#[serde(deserialize_with = "Icon::deserialize")]
	pub icons:     Vec<Icon>,
	pub syntect:   Syntect,
}

impl Theme {
	pub fn new() -> Self {
		let path = BaseDirectories::new().unwrap().get_config_file("yazi/theme.toml");

		let mut parsed: Self = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
		parsed.syntect.theme = absolute_path(&parsed.syntect.theme);
		parsed
	}
}
