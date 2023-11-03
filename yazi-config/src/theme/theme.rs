use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use validator::Validate;
use yazi_shared::expand_path;

use super::{Filetype, Icon, Style};
use crate::{validation::check_validation, MERGED_THEME};

#[derive(Deserialize, Serialize, Validate)]
pub struct Manager {
	cwd: Style,

	// Hovered
	hovered:         Style,
	preview_hovered: Style,

	// Find
	find_keyword:  Style,
	find_position: Style,

	// Marker
	marker_selected: Style,
	marker_copied:   Style,
	marker_cut:      Style,

	// Tab
	tab_active:   Style,
	tab_inactive: Style,
	#[validate(range(min = 1, message = "Must be greater than 0"))]
	tab_width:    u8,

	// Border
	pub border_symbol: String,
	pub border_style:  Style,

	// Offset
	pub(crate) folder_offset:  (u16, u16, u16, u16),
	pub(crate) preview_offset: (u16, u16, u16, u16),

	// Highlighting
	pub syntect_theme: PathBuf,
}

#[derive(Deserialize, Serialize)]
struct Status {
	pub separator_open:  String,
	pub separator_close: String,
	pub separator_style: Style,

	// Mode
	pub mode_normal: Style,
	pub mode_select: Style,
	pub mode_unset:  Style,

	// Progress
	pub progress_label:  Style,
	pub progress_normal: Style,
	pub progress_error:  Style,

	// Permissions
	pub permissions_t: Style,
	pub permissions_r: Style,
	pub permissions_w: Style,
	pub permissions_x: Style,
	pub permissions_s: Style,
}

#[derive(Deserialize, Serialize)]
pub struct Input {
	pub border:   Style,
	pub title:    Style,
	pub value:    Style,
	pub selected: Style,
}

#[derive(Deserialize, Serialize)]
pub struct Select {
	pub border:   Style,
	pub active:   Style,
	pub inactive: Style,
}

#[derive(Deserialize, Serialize)]
pub struct Completion {
	pub border:   Style,
	pub active:   Style,
	pub inactive: Style,

	pub icon_file:    String,
	pub icon_folder:  String,
	pub icon_command: String,
}

#[derive(Deserialize, Serialize)]
pub struct Tasks {
	pub border:  Style,
	pub title:   Style,
	pub hovered: Style,
}

#[derive(Deserialize, Serialize)]
pub struct Which {
	pub mask: Style,
	pub cand: Style,
	pub rest: Style,
	pub desc: Style,

	pub separator:       String,
	pub separator_style: Style,
}

#[derive(Deserialize, Serialize)]
pub struct Help {
	pub on:   Style,
	pub exec: Style,
	pub desc: Style,

	pub hovered: Style,
	pub footer:  Style,
}

#[derive(Deserialize, Serialize)]
pub struct Theme {
	pub manager:    Manager,
	status:         Status,
	pub input:      Input,
	pub select:     Select,
	pub completion: Completion,
	pub tasks:      Tasks,
	pub which:      Which,
	pub help:       Help,

	// File-specific styles
	#[serde(rename = "filetype", deserialize_with = "Filetype::deserialize", skip_serializing)]
	pub filetypes: Vec<Filetype>,
	#[serde(deserialize_with = "Icon::deserialize", skip_serializing)]
	pub icons:     Vec<Icon>,
}

impl Default for Theme {
	fn default() -> Self {
		let mut theme: Self = toml::from_str(&MERGED_THEME).unwrap();

		check_validation(theme.manager.validate());

		theme.manager.syntect_theme = expand_path(&theme.manager.syntect_theme);

		theme
	}
}
