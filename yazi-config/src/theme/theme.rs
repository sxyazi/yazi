use std::{path::PathBuf, str::FromStr};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use validator::Validate;
use yazi_shared::theme::Style;

use super::{Filetype, Flavor, Icons};

#[derive(Deserialize, Serialize)]
pub struct Theme {
	pub flavor:     Flavor,
	pub manager:    Manager,
	mode:           Mode,
	status:         Status,
	pub input:      Input,
	pub confirm:    Confirm,
	pub spot:       Spot,
	pub pick:       Pick,
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
		let theme: Self = toml::from_str(s).context("Failed to parse your theme.toml")?;
		theme.manager.validate()?;
		theme.which.validate()?;

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
struct Mode {
	pub normal_main: Style,
	pub normal_alt:  Style,

	pub select_main: Style,
	pub select_alt:  Style,

	pub unset_main: Style,
	pub unset_alt:  Style,
}

#[derive(Deserialize, Serialize)]
struct Status {
	pub separator_open:  String,
	pub separator_close: String,

	// Progress
	pub progress_label:  Style,
	pub progress_normal: Style,
	pub progress_error:  Style,

	// Permissions
	pub perm_sep:   Style,
	pub perm_type:  Style,
	pub perm_read:  Style,
	pub perm_write: Style,
	pub perm_exec:  Style,
}

#[derive(Deserialize, Serialize)]
pub struct Input {
	pub border:   Style,
	pub title:    Style,
	pub value:    Style,
	pub selected: Style,
}

#[derive(Deserialize, Serialize)]
pub struct Confirm {
	pub border:     Style,
	pub title:      Style,
	pub content:    Style,
	pub list:       Style,
	pub btn_yes:    Style,
	pub btn_no:     Style,
	pub btn_labels: [String; 2],
}

#[derive(Deserialize, Serialize)]
pub struct Spot {
	pub border:   Style,
	pub title:    Style,
	pub headers:  Style,
	pub keys:     Style,
	pub values:   Style,
	pub selected: Style,
}

#[derive(Deserialize, Serialize)]
pub struct Pick {
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
