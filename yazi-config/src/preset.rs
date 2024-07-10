use std::{borrow::Cow, mem, path::{Path, PathBuf}};

use anyhow::{anyhow, Context, Result};
use toml::{Table, Value};

use crate::theme::Flavor;

pub(crate) struct Preset;

impl Preset {
	pub(crate) fn yazi(p: &Path) -> Result<Cow<str>> {
		Self::merge_path(p.join("yazi.toml"), include_str!("../preset/yazi.toml"))
	}

	pub(crate) fn keymap(p: &Path) -> Result<Cow<str>> {
		Self::merge_path(p.join("keymap.toml"), include_str!("../preset/keymap.toml"))
	}

	pub(crate) fn theme(p: &Path) -> Result<Cow<str>> {
		let Ok(user) = std::fs::read_to_string(p.join("theme.toml")) else {
			return Ok(include_str!("../preset/theme.toml").into());
		};
		let Some(use_) = Flavor::parse_use(&user) else {
			return Self::merge_str(&user, include_str!("../preset/theme.toml"));
		};

		let p = p.join(format!("flavors/{use_}.yazi/flavor.toml"));
		let flavor =
			std::fs::read_to_string(&p).with_context(|| anyhow!("Failed to load flavor {p:?}"))?;

		Self::merge_str(&user, &Self::merge_str(&flavor, include_str!("../preset/theme.toml"))?)
	}

	#[inline]
	pub(crate) fn mix<T>(a: &mut Vec<T>, b: Vec<T>, c: Vec<T>) {
		*a = b.into_iter().chain(mem::take(a)).chain(c).collect();
	}

	#[inline]
	pub(crate) fn merge_str(user: &str, base: &str) -> Result<Cow<'static, str>> {
		let mut t = user.parse()?;
		Self::merge(&mut t, base.parse()?, 2);

		Ok(t.to_string().into())
	}

	#[inline]
	fn merge_path(user: PathBuf, base: &str) -> Result<Cow<str>> {
		let s = std::fs::read_to_string(&user).unwrap_or_default();
		if s.is_empty() {
			return Ok(base.into());
		}

		Self::merge_str(&s, base).with_context(|| anyhow!("Loading {user:?}"))
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
