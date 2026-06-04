use std::process::Stdio;

use anyhow::Result;
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::Command,
	sync::mpsc::{self, UnboundedReceiver},
};
use yazi_fs::{File, FsUrl};
use yazi_shared::url::{AsUrl, UrlBuf, UrlLike};
use yazi_vfs::VfsFile;

pub struct RgOpt {
	pub cwd: UrlBuf,
	pub hidden: bool,
	pub subject: String,
	pub args: Vec<String>,
}

pub fn rg(opt: RgOpt) -> Result<UnboundedReceiver<File>> {
	let mut child = Command::new("rg")
	.args(["--color=never", "--no-heading", "--column", "--smart-case"])
		// .args(["--color=never", "--files-with-matches", "--smart-case"])
		.arg(if opt.hidden { "--hidden" } else { "--no-hidden" })
		.args(opt.args)
		.arg(opt.subject)
		.current_dir(&*opt.cwd.as_url().unified_path())
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.spawn()?;

	let mut it = BufReader::new(child.stdout.take().unwrap()).lines();
	let (tx, rx) = mpsc::unbounded_channel();

	tokio::spawn(async move {
		// let mut occurrences_per_file: HashMap<String, Vec<(u32, u32)>> = HashMap::new();
		// let mut current_file_path: String = String::new();
		let mut current_file: String = String::new();
		let mut current_occurrences: Vec<(u32, u32)> = vec![];

		while let Ok(Some(search_line)) = it.next_line().await {
			let Some((file_path, line, col)) = parse_rg_line(&search_line) else { continue };

			if current_file != file_path {
				let Some(url) = build_file_url(opt.cwd.clone(), &current_file, &current_occurrences) else {
					continue;
				};

				if let Ok(file) = File::new(url).await {
					tx.send(file).ok();
				}
				current_file = file_path.clone();
				current_occurrences = vec![];
			}

			current_occurrences.push((line as u32, col as u32));
		}

		if current_occurrences.len() > 0 {
			match build_file_url(opt.cwd, &current_file, &current_occurrences) {
				Some(url) => {
					if let Ok(file) = File::new(url).await {
						tx.send(file).ok();
					}
				}
				None => {}
			}
		}

		child.wait().await.ok();
	});

	Ok(rx)
}

fn build_file_url(cwd: UrlBuf, file_path: &str, occurences: &Vec<(u32, u32)>) -> Option<UrlBuf> {
	let occurrences_str =
		occurences.iter().map(|(line, col)| format!("{line}-{col}")).collect::<Vec<_>>().join(",");
	let url = cwd.try_join(file_path).ok().and_then(|u| u.into_search(occurrences_str).ok());

	url
}

fn parse_rg_line(line: &str) -> Option<(String, usize, usize)> {
	let mut parts = line.split(':');

	let (file, line, col) = (parts.next()?, parts.next()?, parts.next()?);
	if file.is_empty() {
		return None;
	}

	Some((file.to_owned(), line.parse().ok()?, col.parse().ok()?))
}
