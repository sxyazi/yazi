use std::{path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};
use validator::Validate;
use yazi_shared::{fs::expand_path, theme::Style, Xdg};

use super::{Filetype, Flavor, Icons};

#[derive(Deserialize, Serialize)]
pub struct Theme {
	pub flavor:     Flavor,
	pub manager:    Manager,
	status:         Status,
	pub input:      Input,
	pub select:     Select,
	pub completion: Completion,
	pub tasks:      Tasks,
	pub which:      Which,
	pub help:       Help,
	pub notify:     Notify,

	// File-specific styles
	#[serde(rename = "filetype", deserialize_with = "Filetype::deserialize", skip_serializing)]
	pub filetypes: Vec<Filetype>,
	#[serde(rename = "icon", skip_serializing)]
	pub icons:     Icons,
}

impl FromStr for Theme {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut theme: Self = toml::from_str(s)?;
		theme.manager.validate()?;
		theme.which.validate()?;

		if theme.flavor.use_.is_empty() {
			theme.manager.syntect_theme = expand_path(&theme.manager.syntect_theme);
		} else {
			theme.manager.syntect_theme =
				Xdg::config_dir().join(format!("flavors/{}.yazi/tmtheme.xml", theme.flavor.use_));
		}

		Ok(theme)
	}
}

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
	marker_copied:   Style,
	marker_cut:      Style,
	marker_marked:   Style,
	marker_selected: Style,

	// Tab
	tab_active:   Style,
	tab_inactive: Style,
	#[validate(range(min = 1, message = "Must be greater than 0"))]
	tab_width:    u8,

	// Count
	count_copied:   Style,
	count_cut:      Style,
	count_selected: Style,

	// Border
	pub border_symbol: String,
	pub border_style:  Style,

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

#[derive(Deserialize, Serialize, Validate)]
pub struct Which {
	#[validate(range(min = 1, max = 3, message = "Must be between 1 and 3"))]
	pub cols: u8,
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
	pub run:  Style,
	pub desc: Style,

	pub hovered: Style,
	pub footer:  Style,
}

#[derive(Deserialize, Serialize)]
pub struct Notify {
	pub title_info:  Style,
	pub title_warn:  Style,
	pub title_error: Style,

	pub icon_info:  String,
	pub icon_warn:  String,
	pub icon_error: String,
}
