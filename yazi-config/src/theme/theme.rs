use std::path::PathBuf;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use yazi_codegen::{DeserializeOver1, DeserializeOver2};
use yazi_fs::expand_path;
use yazi_shared::theme::Style;

use super::{Filetype, Flavor, Icon};

#[derive(Deserialize, DeserializeOver1, Serialize)]
pub struct Theme {
	pub flavor:  Flavor,
	pub mgr:     Mgr,
	pub tabs:    Tabs,
	pub mode:    Mode,
	pub status:  Status,
	pub which:   Which,
	pub confirm: Confirm,
	pub spot:    Spot,
	pub notify:  Notify,
	pub pick:    Pick,
	pub input:   Input,
	pub cmp:     Cmp,
	pub tasks:   Tasks,
	pub help:    Help,

	// File-specific styles
	#[serde(skip_serializing)]
	pub filetype: Filetype,
	#[serde(skip_serializing)]
	pub icon:     Icon,
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct Mgr {
	cwd: Style,

	// Hovered
	hovered:         Style,
	preview_hovered: Style,

	// Find
	find_keyword:  Style,
	find_position: Style,

	// Symlink
	symlink_target: Style,

	// Marker
	marker_copied:   Style,
	marker_cut:      Style,
	marker_marked:   Style,
	marker_selected: Style,

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

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct Tabs {
	pub active:   Style,
	pub inactive: Style,

	pub sep_inner: TabsSep,
	pub sep_outer: TabsSep,
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct TabsSep {
	pub open:  String,
	pub close: String,
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct Mode {
	pub normal_main: Style,
	pub normal_alt:  Style,

	pub select_main: Style,
	pub select_alt:  Style,

	pub unset_main: Style,
	pub unset_alt:  Style,
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct Status {
	pub overall:   Style,
	pub sep_left:  StatusSep,
	pub sep_right: StatusSep,

	// Permissions
	pub perm_sep:   Style,
	pub perm_type:  Style,
	pub perm_read:  Style,
	pub perm_write: Style,
	pub perm_exec:  Style,

	// Progress
	pub progress_label:  Style,
	pub progress_normal: Style,
	pub progress_error:  Style,
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct StatusSep {
	pub open:  String,
	pub close: String,
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct Which {
	pub cols: u8,
	pub mask: Style,
	pub cand: Style,
	pub rest: Style,
	pub desc: Style,

	pub separator:       String,
	pub separator_style: Style,
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct Confirm {
	pub border:  Style,
	pub title:   Style,
	pub content: Style,
	pub list:    Style,

	pub btn_yes:    Style,
	pub btn_no:     Style,
	pub btn_labels: [String; 2],
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct Spot {
	pub border: Style,
	pub title:  Style,

	pub tbl_col:  Style,
	pub tbl_cell: Style,
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct Notify {
	pub title_info:  Style,
	pub title_warn:  Style,
	pub title_error: Style,

	pub icon_info:  String,
	pub icon_warn:  String,
	pub icon_error: String,
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct Pick {
	pub border:   Style,
	pub active:   Style,
	pub inactive: Style,
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct Input {
	pub border:   Style,
	pub title:    Style,
	pub value:    Style,
	pub selected: Style,
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct Cmp {
	pub border:   Style,
	pub active:   Style,
	pub inactive: Style,

	pub icon_file:    String,
	pub icon_folder:  String,
	pub icon_command: String,
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct Tasks {
	pub border:  Style,
	pub title:   Style,
	pub hovered: Style,
}

#[derive(Deserialize, DeserializeOver2, Serialize)]
pub struct Help {
	pub on:   Style,
	pub run:  Style,
	pub desc: Style,

	pub hovered: Style,
	pub footer:  Style,
}

impl Theme {
	pub(crate) fn reshape(mut self, light: bool) -> Result<Self> {
		if self.which.cols < 1 || self.which.cols > 3 {
			bail!("[which].cols must be between 1 and 3");
		}

		self.icon = self.icon.reshape()?;

		self.mgr.syntect_theme =
			self.flavor.syntect_path(light).unwrap_or_else(|| expand_path(&self.mgr.syntect_theme));

		Ok(self)
	}
}
