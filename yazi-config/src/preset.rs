use std::{mem, path::{Path, PathBuf}};

use toml::{Table, Value};

pub(crate) struct Preset;

impl Preset {
	#[inline]
	pub(crate) fn keymap(dir: &Path) -> String {
		Self::merge_str(dir.join("keymap.toml"), include_str!("../preset/keymap.toml"))
	}

	#[inline]
	pub(crate) fn theme(dir: &Path) -> String {
		Self::merge_str(dir.join("theme.toml"), include_str!("../preset/theme.toml"))
	}

	#[inline]
	pub(crate) fn yazi(dir: &Path) -> String {
		Self::merge_str(dir.join("yazi.toml"), include_str!("../preset/yazi.toml"))
	}

	#[inline]
	pub(crate) fn mix<T>(a: &mut Vec<T>, b: Vec<T>, c: Vec<T>) {
		*a = b.into_iter().chain(mem::take(a)).chain(c).collect();
	}

	fn merge(a: &mut Table, b: Table, max: u8) {
		for (k, v) in b {
			let Some(a) = a.get_mut(&k) else {
				a.insert(k, v);
				continue;
			};

			if max <= 1 {
				continue;
			}

			if let Some(a) = a.as_table_mut() {
				if let Value::Table(b) = v {
					Self::merge(a, b, max - 1);
					continue;
				}
			}
			*a = v;
		}
	}

	fn merge_str(user: PathBuf, base: &str) -> String {
		let mut user = std::fs::read_to_string(user).unwrap_or_default().parse::<Table>().unwrap();
		let base = base.parse::<Table>().unwrap();

		Self::merge(&mut user, base, 2);
		user.to_string()
	}
}
