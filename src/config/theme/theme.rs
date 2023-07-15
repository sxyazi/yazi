use std::{fs, path::PathBuf};

use serde::Deserialize;
use xdg::BaseDirectories;

use super::{ColorGroup, Filetype, Icon, Style};
use crate::misc::absolute_path;

#[derive(Deserialize)]
pub struct Tab {
	pub active:   Style,
	pub inactive: Style,
}

#[derive(Deserialize)]
pub struct Status {
	pub primary:   ColorGroup,
	pub secondary: ColorGroup,
	pub tertiary:  ColorGroup,
	pub body:      ColorGroup,
	pub emphasis:  ColorGroup,
	pub info:      ColorGroup,
	pub success:   ColorGroup,
	pub warning:   ColorGroup,
	pub danger:    ColorGroup,
}

#[derive(Deserialize)]
pub struct Progress {
	pub gauge: Style,
	pub label: Style,
}

#[derive(Deserialize)]
pub struct Selection {
	pub hovered: Style,
}

#[derive(Deserialize)]
pub struct Marker {
	pub selecting: Style,
	pub selected:  Style,
}

#[derive(Deserialize)]
pub struct Preview {
	pub hovered:       Style,
	pub syntect_theme: PathBuf,
}

#[derive(Deserialize)]
pub struct Theme {
	pub tab:       Tab,
	pub status:    Status,
	pub progress:  Progress,
	pub selection: Selection,
	pub marker:    Marker,
	pub preview:   Preview,
	#[serde(deserialize_with = "Filetype::deserialize")]
	pub filetypes: Vec<Filetype>,
	#[serde(deserialize_with = "Icon::deserialize")]
	pub icons:     Vec<Icon>,
}

impl Theme {
	pub fn new() -> Self {
		let path = BaseDirectories::new().unwrap().get_config_file("yazi/theme.toml");

		let mut parsed: Self = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
		parsed.preview.syntect_theme = absolute_path(&parsed.preview.syntect_theme);
		parsed
	}
}
