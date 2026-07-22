use std::path::PathBuf;

use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use serde::{Deserialize, Deserializer, de};
use yazi_binding::style::StyleFlat;
use yazi_codegen::{DeserializeOver, DeserializeOver1, DeserializeOver2, Overlay};
use yazi_fs::{Xdg, ok_or_not_found, path::sanitize_path};
use yazi_shim::{arc_swap::IntoPointee, cell::SyncCell};

use super::{Custom, Filetype, Flavor, Icon};
use crate::YAZI;

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

	// User-defined custom sections
	#[serde(flatten, default)]
	pub custom: Custom,
}

impl Theme {
	pub(crate) fn read() -> Result<(PathBuf, String)> {
		let p = Xdg::config_dir().join("theme.toml");
		let s = ok_or_not_found(std::fs::read_to_string(&p))
			.with_context(|| format!("Failed to read theme {p:?}"))?;
		Ok((p, s))
	}

	// FIXME: remove
	pub(crate) fn reshape(mut self, light: bool) -> Result<Self> {
		if let Some(p) = self.flavor.syntect_path(light) {
			self.mgr.syntect_theme = p.into_pointee();
		}

		Ok(self)
	}
}

// --- App
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct App {
	pub overall: SyncCell<StyleFlat>,
}

// --- Mgr
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Mgr {
	pub cwd: SyncCell<StyleFlat>,

	// Find
	pub find_keyword:  SyncCell<StyleFlat>,
	pub find_position: SyncCell<StyleFlat>,

	// Symlink
	pub symlink_target: SyncCell<StyleFlat>,

	// Marker
	pub marker_copied:   SyncCell<StyleFlat>,
	pub marker_cut:      SyncCell<StyleFlat>,
	pub marker_marked:   SyncCell<StyleFlat>,
	pub marker_selected: SyncCell<StyleFlat>,
	pub marker_symbol:   ArcSwap<String>,

	// Count
	pub count_copied:   SyncCell<StyleFlat>,
	pub count_cut:      SyncCell<StyleFlat>,
	pub count_selected: SyncCell<StyleFlat>,

	// Border
	pub border_symbol: ArcSwap<String>,
	pub border_style:  SyncCell<StyleFlat>,

	// Highlighting
	#[serde(deserialize_with = "deserialize_syntect_theme")]
	pub syntect_theme: ArcSwap<PathBuf>,
}

// --- Tabs
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Tabs {
	pub active:   SyncCell<StyleFlat>,
	pub inactive: SyncCell<StyleFlat>,

	pub sep_inner: ArcSwap<TabsSep>,
	pub sep_outer: ArcSwap<TabsSep>,
}

#[derive(Deserialize)]
pub struct TabsSep {
	pub open:  String,
	pub close: String,
}

// --- Mode
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Mode {
	pub normal_main: SyncCell<StyleFlat>,
	pub normal_alt:  SyncCell<StyleFlat>,

	pub select_main: SyncCell<StyleFlat>,
	pub select_alt:  SyncCell<StyleFlat>,

	pub unset_main: SyncCell<StyleFlat>,
	pub unset_alt:  SyncCell<StyleFlat>,
}

// --- Indicator
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Indicator {
	pub parent:  SyncCell<StyleFlat>,
	pub current: SyncCell<StyleFlat>,
	pub preview: SyncCell<StyleFlat>,
	pub padding: ArcSwap<IndicatorPadding>,
}

#[derive(Deserialize)]
pub struct IndicatorPadding {
	pub open:  String,
	pub close: String,
}

// --- Status
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Status {
	pub overall:   SyncCell<StyleFlat>,
	pub sep_left:  ArcSwap<StatusSep>,
	pub sep_right: ArcSwap<StatusSep>,

	// Permissions
	pub perm_sep:   SyncCell<StyleFlat>,
	pub perm_type:  SyncCell<StyleFlat>,
	pub perm_read:  SyncCell<StyleFlat>,
	pub perm_write: SyncCell<StyleFlat>,
	pub perm_exec:  SyncCell<StyleFlat>,

	// Progress
	pub progress_label:  SyncCell<StyleFlat>,
	pub progress_normal: SyncCell<StyleFlat>,
	pub progress_error:  SyncCell<StyleFlat>,
}

#[derive(Deserialize)]
pub struct StatusSep {
	pub open:  String,
	pub close: String,
}

// --- Which
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Which {
	#[serde(deserialize_with = "deserialize_which_cols")]
	pub cols: SyncCell<u8>,
	pub mask: SyncCell<StyleFlat>,
	pub cand: SyncCell<StyleFlat>,
	pub rest: SyncCell<StyleFlat>,
	pub desc: SyncCell<StyleFlat>,

	pub separator:       ArcSwap<String>,
	pub separator_style: SyncCell<StyleFlat>,
}

// --- Confirm
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Confirm {
	pub border: SyncCell<StyleFlat>,
	pub title:  SyncCell<StyleFlat>,
	pub body:   SyncCell<StyleFlat>,
	pub list:   SyncCell<StyleFlat>,

	pub btn_yes:    SyncCell<StyleFlat>,
	pub btn_no:     SyncCell<StyleFlat>,
	pub btn_labels: ArcSwap<[String; 2]>,
}

// --- Spot
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Spot {
	pub border: SyncCell<StyleFlat>,
	pub title:  SyncCell<StyleFlat>,

	pub tbl_col:  SyncCell<StyleFlat>,
	pub tbl_cell: SyncCell<StyleFlat>,
}

// --- Notify
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Notify {
	pub title_info:  SyncCell<StyleFlat>,
	pub title_warn:  SyncCell<StyleFlat>,
	pub title_error: SyncCell<StyleFlat>,

	pub icon_info:  ArcSwap<String>,
	pub icon_warn:  ArcSwap<String>,
	pub icon_error: ArcSwap<String>,
}

// --- Pick
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Pick {
	pub border:   SyncCell<StyleFlat>,
	pub active:   SyncCell<StyleFlat>,
	pub inactive: SyncCell<StyleFlat>,
}

// --- Input
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Input {
	pub border:   SyncCell<StyleFlat>,
	pub title:    SyncCell<StyleFlat>,
	pub value:    SyncCell<StyleFlat>,
	pub selected: SyncCell<StyleFlat>,
}

impl From<&Input> for yazi_widgets::input::InputStyles {
	fn from(input: &Input) -> Self {
		Self {
			normal:   Some(input.value.get().into()),
			selected: Some(input.selected.get().into()),
			blink:    Some(YAZI.input.cursor_blink),
		}
	}
}

// --- Cmp
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Cmp {
	pub border:   SyncCell<StyleFlat>,
	pub active:   SyncCell<StyleFlat>,
	pub inactive: SyncCell<StyleFlat>,

	pub icon_file:    ArcSwap<String>,
	pub icon_folder:  ArcSwap<String>,
	pub icon_command: ArcSwap<String>,
}

// --- Tasks
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Tasks {
	pub border:  SyncCell<StyleFlat>,
	pub title:   SyncCell<StyleFlat>,
	pub hovered: SyncCell<StyleFlat>,
}

// --- Help
#[derive(Deserialize, DeserializeOver, DeserializeOver2, Overlay)]
pub struct Help {
	pub border:  SyncCell<StyleFlat>,
	pub chord:   SyncCell<StyleFlat>,
	pub action:  SyncCell<StyleFlat>,
	pub hovered: SyncCell<StyleFlat>,
}

fn deserialize_syntect_theme<'de, D>(deserializer: D) -> Result<ArcSwap<PathBuf>, D::Error>
where
	D: Deserializer<'de>,
{
	let mut path = PathBuf::deserialize(deserializer)?;
	if !path.as_os_str().is_empty() {
		path = sanitize_path(path).ok_or_else(|| {
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
