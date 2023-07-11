use std::{collections::VecDeque, env, path::{Path, PathBuf}};

use anyhow::Result;
use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};
use tokio::{fs::{self, File}, io::{self, AsyncBufReadExt, BufReader}, select, sync::{mpsc, oneshot}, time};

#[inline]
pub fn tty_size() -> winsize {
	unsafe {
		let s: winsize = std::mem::zeroed();
		ioctl(STDOUT_FILENO, TIOCGWINSZ, &s);
		s
	}
}

#[inline]
pub fn tty_ratio() -> (f64, f64) {
	let s = tty_size();
	(f64::from(s.ws_xpixel) / f64::from(s.ws_col), f64::from(s.ws_ypixel) / f64::from(s.ws_row))
}

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

pub async fn first_n_lines(path: &Path, n: usize) -> Result<Vec<String>> {
	let mut lines = Vec::new();
	let mut it = BufReader::new(File::open(path).await?).lines();
	for _ in 0..n {
		if let Some(line) = it.next_line().await? {
			lines.push(line);
		} else {
			break;
		}
	}
	Ok(lines)
}

pub async fn calculate_size(path: &Path) -> u64 {
	let mut total = 0;
	let mut stack = VecDeque::from([path.to_path_buf()]);
	while let Some(path) = stack.pop_front() {
		let meta = if let Ok(meta) = fs::symlink_metadata(&path).await {
			meta
		} else {
			continue;
		};

		if !meta.is_dir() {
			total += meta.len();
			continue;
		}

		let mut it = if let Ok(it) = fs::read_dir(path).await {
			it
		} else {
			continue;
		};

		while let Ok(Some(entry)) = it.next_entry().await {
			let meta = if let Ok(m) = entry.metadata().await {
				m
			} else {
				continue;
			};

			if meta.is_dir() {
				stack.push_back(entry.path());
			} else {
				total += meta.len();
			}
		}
	}
	total
}

pub fn copy_with_progress(from: &Path, to: &Path) -> mpsc::Receiver<Result<u64, io::Error>> {
	let (tx, rx) = mpsc::channel(1);
	let (tick_tx, mut tick_rx) = oneshot::channel();

	tokio::spawn({
		let (from, to) = (from.to_path_buf(), to.to_path_buf());

		async move {
			let _ = match fs::copy(from, to).await {
				Ok(len) => tick_tx.send(Ok(len)),
				Err(e) => tick_tx.send(Err(e)),
			};
		}
	});

	tokio::spawn({
		let tx = tx.clone();
		let to = to.to_path_buf();

		async move {
			let mut last = 0;
			let mut exit = None;
			loop {
				select! {
					res = &mut tick_rx => exit = Some(res.unwrap()),
					_ = tx.closed() => break,
					_ = time::sleep(time::Duration::from_secs(1)) => (),
				}

				match exit {
					Some(Ok(len)) => {
						if len > last {
							tx.send(Ok(len - last)).await.ok();
						}
						tx.send(Ok(0)).await.ok();
						break;
					}
					Some(Err(e)) => {
						tx.send(Err(e)).await.ok();
						break;
					}
					None => {}
				}

				let len = fs::symlink_metadata(&to).await.map(|m| m.len()).unwrap_or(0);
				if len > last {
					tx.send(Ok(len - last)).await.ok();
					last = len;
				}
			}
		}
	});

	rx
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
