use std::{fs, mem};

use toml::Table;

use crate::BOOT;

pub(crate) struct Preset;

impl Preset {
	#[inline]
	pub(crate) fn keymap() -> String {
		Self::merge_str("keymap.toml", include_str!("../preset/keymap.toml"))
	}

	#[inline]
	pub(crate) fn theme() -> String {
		Self::merge_str("theme.toml", include_str!("../preset/theme.toml"))
	}

	#[inline]
	pub(crate) fn yazi() -> String {
		Self::merge_str("yazi.toml", include_str!("../preset/yazi.toml"))
	}

	#[inline]
	pub(crate) fn mix<T>(a: &mut Vec<T>, b: Vec<T>, c: Vec<T>) {
		*a = b.into_iter().chain(mem::take(a)).chain(c).collect();
	}

	fn merge(a: &mut Table, b: &Table, max: u8) {
		for (k, v) in b {
			let Some(a) = a.get_mut(k) else {
				a.insert(k.clone(), v.clone());
				continue;
			};

			if max <= 1 {
				continue;
			}

			if let Some(a) = a.as_table_mut() {
				if let Some(b) = v.as_table() {
					Self::merge(a, b, max - 1);
					continue;
				}
			}
			*a = v.clone();
		}
	}

	fn merge_str(user: &str, base: &str) -> String {
		let path = BOOT.config_dir.join(user);
		let mut user = fs::read_to_string(path).unwrap_or_default().parse::<Table>().unwrap();

		let base = base.parse::<Table>().unwrap();
		Self::merge(&mut user, &base, 2);
		user.to_string()
	}
}
