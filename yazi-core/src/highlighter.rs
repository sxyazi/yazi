use std::{fs::File, io::BufReader, sync::OnceLock};

use anyhow::Result;
use syntect::{dumps::from_uncompressed_data, highlighting::{Theme, ThemeSet}, parsing::SyntaxSet};
use yazi_config::THEME;

static SYNTECT_SYNTAX: OnceLock<SyntaxSet> = OnceLock::new();
static SYNTECT_THEME: OnceLock<Theme> = OnceLock::new();

#[inline]
pub fn highlighter() -> (&'static SyntaxSet, &'static Theme) {
	let syntaxes =
		SYNTECT_SYNTAX.get_or_init(|| from_uncompressed_data(yazi_prebuild::syntaxes()).unwrap());

	let theme = SYNTECT_THEME.get_or_init(|| {
		let from_file = || -> Result<Theme> {
			let file = File::open(&THEME.manager.syntect_theme)?;
			Ok(ThemeSet::load_from_reader(&mut BufReader::new(file))?)
		};
		from_file().unwrap_or_else(|_| ThemeSet::load_defaults().themes["base16-ocean.dark"].clone())
	});

	(syntaxes, theme)
}
