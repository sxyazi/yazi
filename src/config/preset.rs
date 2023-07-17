use std::fs;

use toml::Table;
use xdg::BaseDirectories;

pub(crate) struct Preset;

impl Preset {
	fn merge(a: &mut Table, b: &Table, max: u8) {
		for (k, v) in b {
			let a = if let Some(a) = a.get_mut(k) {
				a
			} else {
				a.insert(k.clone(), v.clone());
				continue;
			};

			if k == "icons" {
				continue;
			}

			if max - 1 > 0 {
				if let Some(b) = v.as_table() {
					if let Some(a) = a.as_table_mut() {
						Self::merge(a, b, max - 1);
						continue;
					}
				}
			}
			*a = v.clone();
		}
	}

	fn merge_str(base: &str, user: &str) -> String {
		let path = BaseDirectories::new().unwrap().get_config_file(user);
		let user = fs::read_to_string(path).unwrap_or("".to_string()).parse::<Table>().unwrap();

		let mut base = base.parse::<Table>().unwrap();
		Self::merge(&mut base, &user, 2);
		base.to_string()
	}

	#[inline]
	pub(crate) fn keymap() -> String {
		Self::merge_str(include_str!("../../config/keymap.toml"), "yazi/keymap.toml")
	}

	#[inline]
	pub(crate) fn theme() -> String {
		Self::merge_str(include_str!("../../config/theme.toml"), "yazi/theme.toml")
	}

	#[inline]
	pub(crate) fn yazi() -> String {
		Self::merge_str(include_str!("../../config/yazi.toml"), "yazi/yazi.toml")
	}
}
