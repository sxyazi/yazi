use std::path::PathBuf;

use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use serde::{Deserialize, Deserializer, de};
use yazi_codegen::{DeserializeOver, DeserializeOver1, DeserializeOver2, Overlay};
use yazi_fs::{Xdg, ok_or_not_found};
use yazi_shim::{arc_swap::IntoPointee, cell::SyncCell};

use super::{Filetype, Flavor, Icon};
use crate::{Style, normalize_path};

#[derive(Deserialize, DeserializeOver, DeserializeOver1, Overlay)]
pub struct Theme {
	pub flavor:    Flavor,
	pub app:       App,
	pub mgr:       Mgr,
	pub tabs:      Tabs,
	pub mode:      Mode,
	pub indicator: Indicator,
	pub status:    Status,
	pub which:     Which,
	pub confirm:   Confirm,
	pub spot:      Spot,
	pub notify:    Notify,
	pub pick:      Pick,
	pub input:     Input,
	pub cmp:       Cmp,
	pub tasks:     Tasks,
	pub help:      Help,

	// File-specific styles
	pub filetype: Filetype,
	pub icon:     Icon,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct App {
	pub overall: SyncCell<Style>,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Mgr {
	pub cwd: SyncCell<Style>,

	// Find
	pub find_keyword:  SyncCell<Style>,
	pub find_position: SyncCell<Style>,

	// Symlink
	pub symlink_target: SyncCell<Style>,

	// Marker
	pub marker_copied:   SyncCell<Style>,
	pub marker_cut:      SyncCell<Style>,
	pub marker_marked:   SyncCell<Style>,
	pub marker_selected: SyncCell<Style>,
	pub marker_symbol:   ArcSwap<String>,

	// Count
	pub count_copied:   SyncCell<Style>,
	pub count_cut:      SyncCell<Style>,
	pub count_selected: SyncCell<Style>,

	// Border
	pub border_symbol: ArcSwap<String>,
	pub border_style:  SyncCell<Style>,

	// Highlighting
	#[serde(deserialize_with = "deserialize_syntect_theme")]
	pub syntect_theme: ArcSwap<PathBuf>,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Tabs {
	pub active:   SyncCell<Style>,
	pub inactive: SyncCell<Style>,

	pub sep_inner: ArcSwap<TabsSep>,
	pub sep_outer: ArcSwap<TabsSep>,
}

#[derive(Deserialize)]
pub struct TabsSep {
	pub open:  String,
	pub close: String,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Mode {
	pub normal_main: SyncCell<Style>,
	pub normal_alt:  SyncCell<Style>,

	pub select_main: SyncCell<Style>,
	pub select_alt:  SyncCell<Style>,

	pub unset_main: SyncCell<Style>,
	pub unset_alt:  SyncCell<Style>,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Indicator {
	pub parent:  SyncCell<Style>,
	pub current: SyncCell<Style>,
	pub preview: SyncCell<Style>,
	pub padding: ArcSwap<IndicatorPadding>,
}

#[derive(Deserialize)]
pub struct IndicatorPadding {
	pub open:  String,
	pub close: String,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Status {
	pub overall:   SyncCell<Style>,
	pub sep_left:  ArcSwap<StatusSep>,
	pub sep_right: ArcSwap<StatusSep>,

	// Permissions
	pub perm_sep:   SyncCell<Style>,
	pub perm_type:  SyncCell<Style>,
	pub perm_read:  SyncCell<Style>,
	pub perm_write: SyncCell<Style>,
	pub perm_exec:  SyncCell<Style>,

	// Progress
	pub progress_label:  SyncCell<Style>,
	pub progress_normal: SyncCell<Style>,
	pub progress_error:  SyncCell<Style>,
}

#[derive(Deserialize)]
pub struct StatusSep {
	pub open:  String,
	pub close: String,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Which {
	#[serde(deserialize_with = "deserialize_which_cols")]
	pub cols: SyncCell<u8>,
	pub mask: SyncCell<Style>,
	pub cand: SyncCell<Style>,
	pub rest: SyncCell<Style>,
	pub desc: SyncCell<Style>,

	pub separator:       ArcSwap<String>,
	pub separator_style: SyncCell<Style>,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Confirm {
	pub border: SyncCell<Style>,
	pub title:  SyncCell<Style>,
	pub body:   SyncCell<Style>,
	pub list:   SyncCell<Style>,

	pub btn_yes:    SyncCell<Style>,
	pub btn_no:     SyncCell<Style>,
	pub btn_labels: ArcSwap<[String; 2]>,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Spot {
	pub border: SyncCell<Style>,
	pub title:  SyncCell<Style>,

	pub tbl_col:  SyncCell<Style>,
	pub tbl_cell: SyncCell<Style>,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Notify {
	pub title_info:  SyncCell<Style>,
	pub title_warn:  SyncCell<Style>,
	pub title_error: SyncCell<Style>,

	pub icon_info:  ArcSwap<String>,
	pub icon_warn:  ArcSwap<String>,
	pub icon_error: ArcSwap<String>,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Pick {
	pub border:   SyncCell<Style>,
	pub active:   SyncCell<Style>,
	pub inactive: SyncCell<Style>,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Input {
	pub border:   SyncCell<Style>,
	pub title:    SyncCell<Style>,
	pub value:    SyncCell<Style>,
	pub selected: SyncCell<Style>,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Cmp {
	pub border:   SyncCell<Style>,
	pub active:   SyncCell<Style>,
	pub inactive: SyncCell<Style>,

	pub icon_file:    ArcSwap<String>,
	pub icon_folder:  ArcSwap<String>,
	pub icon_command: ArcSwap<String>,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Tasks {
	pub border:  SyncCell<Style>,
	pub title:   SyncCell<Style>,
	pub hovered: SyncCell<Style>,
}

#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Help {
	pub on:   SyncCell<Style>,
	pub run:  SyncCell<Style>,
	pub desc: SyncCell<Style>,

	pub hovered: SyncCell<Style>,
	pub footer:  SyncCell<Style>,
}

impl Theme {
	pub(crate) fn read() -> Result<String> {
		let p = Xdg::config_dir().join("theme.toml");
		ok_or_not_found(std::fs::read_to_string(&p))
			.with_context(|| format!("Failed to read theme {p:?}"))
	}

	// FIXME: remove
	pub(crate) fn reshape(mut self, light: bool) -> Result<Self> {
		if let Some(p) = self.flavor.syntect_path(light) {
			self.mgr.syntect_theme = p.into_pointee();
		}

		Ok(self)
	}
}

impl App {
	pub fn bg_color(&self) -> String {
		self.overall.get().bg.map(|c| c.to_string()).unwrap_or_default()
	}
}

fn deserialize_syntect_theme<'de, D>(deserializer: D) -> Result<ArcSwap<PathBuf>, D::Error>
where
	D: Deserializer<'de>,
{
	let mut path = PathBuf::deserialize(deserializer)?;
	if !path.as_os_str().is_empty() {
		path = normalize_path(path).ok_or_else(|| {
			de::Error::custom("syntect_theme must be either empty or an absolute path.")
		})?;
	}

	Ok(path.into_pointee())
}

fn deserialize_which_cols<'de, D>(deserializer: D) -> Result<SyncCell<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let cols = u8::deserialize(deserializer)?;
	if (1..=3).contains(&cols) {
		Ok(SyncCell::new(cols))
	} else {
		Err(de::Error::custom("cols must be between 1 and 3"))
	}
}
