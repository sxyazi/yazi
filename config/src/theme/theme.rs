use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use shared::expand_path;
use validator::Validate;

use super::{Files, Filetype, Icon, Marker, Status, Style};
use crate::{validation::check_validation, MERGED_THEME};

#[derive(Deserialize, Serialize, Validate)]
pub struct Tabs {
	pub active:    Style,
	pub inactive:  Style,
	pub header:    Style,
	#[validate(range(min = 1, message = "Must be greater than 0"))]
	pub max_width: u8,
}

#[derive(Deserialize, Serialize)]
pub struct Preview {
	pub hovered:       Style,
	pub syntect_theme: PathBuf,
}

#[derive(Deserialize, Serialize)]
pub struct Theme {
	pub tabs:      Tabs,
	pub status:    Status,
	pub files:     Files,
	pub marker:    Marker,
	pub preview:   Preview,
	#[serde(rename = "filetype", deserialize_with = "Filetype::deserialize", skip_serializing)]
	pub filetypes: Vec<Filetype>,
	#[serde(deserialize_with = "Icon::deserialize", skip_serializing)]
	pub icons:     Vec<Icon>,
}

impl Default for Theme {
	fn default() -> Self {
		let mut theme: Self = toml::from_str(&MERGED_THEME).unwrap();

		check_validation(theme.tabs.validate());

		theme.preview.syntect_theme = expand_path(&theme.preview.syntect_theme);

		theme
	}
}
