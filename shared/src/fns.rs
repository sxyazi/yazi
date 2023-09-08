use std::{env, path::{Path, PathBuf}};

use tokio::fs;

use crate::Url;

pub fn expand_path(p: impl AsRef<Path>) -> PathBuf {
	let p = p.as_ref();
	if let Ok(p) = p.strip_prefix("~") {
		if let Some(home) = env::var_os("HOME") {
			return PathBuf::from_iter([&home, p.as_os_str()]);
		}
	}
	p.to_path_buf()
}

#[inline]
pub fn expand_url(mut u: Url) -> Url {
	u.set_path(expand_path(&u));
	u
}

pub fn short_path(p: &Path, base: &Path) -> String {
	if let Ok(p) = p.strip_prefix(base) {
		return p.display().to_string();
	}
	p.display().to_string()
}

pub fn readable_path(p: &Path) -> String {
	if let Ok(home) = env::var("HOME") {
		if let Ok(p) = p.strip_prefix(home) {
			return format!("~/{}", p.display());
		}
	}
	p.display().to_string()
}

pub fn readable_size(size: u64) -> String {
	let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
	let mut size = size as f64;
	let mut i = 0;
	while size > 1024.0 && i < units.len() - 1 {
		size /= 1024.0;
		i += 1;
	}
	format!("{:.1} {}", size, units[i])
}

pub async fn unique_path(mut p: Url) -> Url {
	let Some(name) = p.file_name().map(|n| n.to_os_string()) else {
		return p;
	};

	let mut i = 0;
	while fs::symlink_metadata(&p).await.is_ok() {
		i += 1;
		let mut name = name.clone();
		name.push(format!("_{i}"));
		p.set_file_name(name);
	}
	p
}

#[inline]
pub fn optional_bool(s: &str) -> Option<bool> {
	if s == "true" {
		Some(true)
	} else if s == "false" {
		Some(false)
	} else {
		None
	}
}
