use std::{mem, path::{Path, PathBuf}};

use anyhow::Context;
use toml::{Table, Value};

use crate::theme::Flavor;

pub(crate) struct Preset;

impl Preset {
	pub(crate) fn yazi(p: &Path) -> String {
		Self::merge_path(p.join("yazi.toml"), include_str!("../preset/yazi.toml"))
	}

	pub(crate) fn keymap(p: &Path) -> String {
		Self::merge_path(p.join("keymap.toml"), include_str!("../preset/keymap.toml"))
	}

	pub(crate) fn theme(p: &Path) -> String {
		let Ok(user) = std::fs::read_to_string(p.join("theme.toml")) else {
			return include_str!("../preset/theme.toml").to_owned();
		};
		let Some(use_) = Flavor::parse_use(&user) else {
			return Self::merge_str(&user, include_str!("../preset/theme.toml"));
		};

		let p = p.join(format!("flavors/{}.yazi/flavor.toml", use_));
		let flavor = std::fs::read_to_string(&p)
			.with_context(|| format!("Failed to load flavor {:?}", p))
			.unwrap();

		Self::merge_str(&user, &Self::merge_str(&flavor, include_str!("../preset/theme.toml")))
	}

	#[inline]
	pub(crate) fn mix<T>(a: &mut Vec<T>, b: Vec<T>, c: Vec<T>) {
		*a = b.into_iter().chain(mem::take(a)).chain(c).collect();
	}

	#[inline]
	pub(crate) fn merge_str(user: &str, base: &str) -> String {
		let mut t = user.parse().unwrap();
		Self::merge(&mut t, base.parse().unwrap(), 2);

		t.to_string()
	}

	#[inline]
	fn merge_path(user: PathBuf, base: &str) -> String {
		let s = std::fs::read_to_string(user).unwrap_or_default();
		if s.is_empty() {
			return base.to_string();
		}

		Self::merge_str(&s, base)
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
}
