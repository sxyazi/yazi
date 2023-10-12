use serde::{Deserialize, Serialize};
use shared::expand_path;
use validator::Validate;

use super::{Files, Filetype, Help, Icon, Input, Marker, Preview, Select, Status, Tabs, Tasks, Which};
use crate::{validation::check_validation, MERGED_THEME};

#[derive(Deserialize, Serialize)]
pub struct Theme {
	// Status
	pub status: Status,

	// Manager
	pub tabs:    Tabs,
	pub files:   Files,
	pub marker:  Marker,
	pub preview: Preview,

	// File-specific styles
	#[serde(rename = "filetype", deserialize_with = "Filetype::deserialize", skip_serializing)]
	pub filetypes: Vec<Filetype>,
	#[serde(deserialize_with = "Icon::deserialize", skip_serializing)]
	pub icons:     Vec<Icon>,

	// Input
	pub input: Input,

	// Select
	pub select: Select,

	// Tasks
	pub tasks: Tasks,

	// Which
	pub which: Which,

	// Help
	pub help: Help,
}

impl Default for Theme {
	fn default() -> Self {
		let mut theme: Self = toml::from_str(&MERGED_THEME).unwrap();

		check_validation(theme.tabs.validate());

		theme.preview.syntect_theme = expand_path(&theme.preview.syntect_theme);

		theme
	}
}
