use std::path::PathBuf;

use serde::Deserialize;
use shared::absolute_path;

use super::{ColorGroup, Filetype, Icon, Style};
use crate::MERGED_THEME;

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
	pub separator: SectionSeparator,
}

#[derive(Deserialize)]
pub struct SectionSeparator {
	pub opening: String,
	pub closing: String,
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
	#[serde(rename = "filetype", deserialize_with = "Filetype::deserialize")]
	pub filetypes: Vec<Filetype>,
	#[serde(deserialize_with = "Icon::deserialize")]
	pub icons:     Vec<Icon>,
}

impl Theme {
	pub fn new() -> Self {
		let mut theme: Self = toml::from_str(&MERGED_THEME).unwrap();
		theme.preview.syntect_theme =
			futures::executor::block_on(absolute_path(&theme.preview.syntect_theme));
		theme
	}
}
