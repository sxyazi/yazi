use std::{env, path::{Path, PathBuf}};

use tokio::fs;

pub fn absolute_path(p: &Path) -> PathBuf {
	if p.starts_with("~") {
		if let Ok(home) = env::var("HOME") {
			let mut expanded = PathBuf::new();
			expanded.push(home);
			expanded.push(p.strip_prefix("~").unwrap());
			return expanded;
		}
	}
	p.to_path_buf()
}

pub fn readable_path(p: &Path, base: &Path) -> String {
	if let Ok(p) = p.strip_prefix(base) {
		return p.display().to_string();
	}
	p.display().to_string()
}

pub fn readable_home(p: &Path) -> String {
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

pub async fn unique_path(mut p: PathBuf) -> PathBuf {
	let name = if let Some(name) = p.file_name() {
		name.to_os_string()
	} else {
		return p;
	};

	let mut i = 0;
	while fs::symlink_metadata(&p).await.is_ok() {
		i += 1;
		let mut name = name.clone();
		name.push(format!("_{}", i));
		p.set_file_name(name);
	}
	p
}

#[inline]
pub fn optinal_bool(s: &str) -> Option<bool> {
	if s == "true" {
		Some(true)
	} else if s == "false" {
		Some(false)
	} else {
		None
	}
}

pub fn valid_mimetype(str: &str) -> bool {
	let parts = str.split('/').collect::<Vec<_>>();
	if parts.len() != 2 {
		return false;
	}

	let b = match parts[0] {
		"application" => true,
		"audio" => true,
		"example" => true,
		"font" => true,
		"image" => true,
		"message" => true,
		"model" => true,
		"multipart" => true,
		"text" => true,
		"video" => true,
		_ => false,
	};
	b && !parts[1].is_empty()
}
