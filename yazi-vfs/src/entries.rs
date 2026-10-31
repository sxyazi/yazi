use std::io;

use tokio::{join, select, sync::mpsc::{self, UnboundedReceiver}};
use yazi_fs::{Entries, engine::{DirReader, FileHolder}, file::{File, Files}, mounts::PARTITIONS, stat::StatKind};
use yazi_shared::url::{UrlBuf, UrlLike};

use crate::engine::{self, DirEntry};

pub trait VfsEntries {
	fn from_dir(dir: &UrlBuf) -> impl Future<Output = io::Result<UnboundedReceiver<File>>>;

	fn from_dir_bulk(dir: &UrlBuf) -> impl Future<Output = io::Result<Files>>;

	fn revalidate(file: &File) -> impl Future<Output = io::Result<Option<File>>>;
}

pub async fn read_dot_hidden(dir: &UrlBuf) -> Option<hashbrown::HashSet<std::ffi::OsString>> {
	let path = dir.as_local()?;
	let content = tokio::fs::read(path.join(".hidden")).await.ok()?;
	let mut set = hashbrown::HashSet::new();
	#[cfg(unix)]
	{
		use std::os::unix::ffi::OsStrExt;
		for line in content.split(|&b| b == b'\n') {
			let line = line.strip_suffix(b"\r").unwrap_or(line);
			let line = line.strip_suffix(b" ").unwrap_or(line);
			if !line.is_empty() && !line.starts_with(b"#") {
				set.insert(std::ffi::OsStr::from_bytes(line).to_os_string());
			}
		}
	}
	#[cfg(not(unix))]
	{
		if let Ok(s) = String::from_utf8(content) {
			for line in s.lines() {
				let line = line.trim_end_matches(['\r', ' ']);
				if !line.is_empty() && !line.starts_with('#') {
					set.insert(line.into());
				}
			}
		}
	}
	Some(set)
}

pub fn apply_dot_hidden(file: &mut File, hidden: Option<&hashbrown::HashSet<std::ffi::OsString>>) {
	if let Some(hidden) = hidden
		&& let Some(name) = file.name()
		&& let Ok(os) = name.as_os()
		&& hidden.contains(os)
	{
		file.stat.kind |= StatKind::HIDDEN;
	}
}

impl VfsEntries for Entries {
	async fn from_dir(dir: &UrlBuf) -> std::io::Result<UnboundedReceiver<File>> {
		let mut it = engine::read_dir(dir).await?;
		let (tx, rx) = mpsc::unbounded_channel();
		let hidden = read_dot_hidden(dir).await;

		tokio::spawn(async move {
			loop {
				let future = async {
					let dent = it.next().await.ok().flatten()?;
					let mut file = match dent.file().await {
						Ok(file) => file,
						Err(_) => File::from_dummy(dent.url(), dent.file_type().await.ok()),
					};
					apply_dot_hidden(&mut file, hidden.as_ref());
					Some(file)
				};

				select! {
					biased;
					_ = tx.closed() => break,
					file = future => {
						let Some(file) = file else { break };
						if tx.send(file).is_err() {
							break;
						}
					}
				}
			}
		});
		Ok(rx)
	}

	async fn from_dir_bulk(dir: &UrlBuf) -> io::Result<Files> {
		let mut it = engine::read_dir(dir).await?;
		let hidden = read_dot_hidden(dir).await;
		let mut entries = Vec::new();
		while let Ok(Some(dent)) = it.next().await {
			entries.push(dent);
		}

		let (first, rest) = entries.split_at(entries.len() / 3);
		let (second, third) = rest.split_at(entries.len() / 3);
		async fn go(
			entries: &[DirEntry],
			hidden: Option<&hashbrown::HashSet<std::ffi::OsString>>,
		) -> Files {
			let mut files = Files::with_capacity(entries.len());
			for dent in entries {
				let mut file = match dent.file().await {
					Ok(file) => file,
					Err(_) => File::from_dummy(dent.url(), dent.file_type().await.ok()),
				};
				apply_dot_hidden(&mut file, hidden);
				files.push(file);
			}
			files
		}

		let (first, second, third) =
			join!(go(first, hidden.as_ref()), go(second, hidden.as_ref()), go(third, hidden.as_ref()));
		Ok([first, second, third].into_iter().flatten().collect())
	}

	async fn revalidate(old: &File) -> io::Result<Option<File>> {
		match engine::revalidate(old).await? {
			Some(new) if new.url != old.url => {
				Err(io::Error::new(io::ErrorKind::InvalidData, "revalidated file URL changed"))
			}
			Some(new) if !new.is_dir() => Err(io::ErrorKind::NotADirectory.into()),
			Some(new) => Ok(Some(new)),
			None if PARTITIONS.read().timeless(old.stat) => Ok(Some(old.clone())),
			None => Ok(None),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::fs;

	use super::*;

	#[tokio::test]
	async fn test_dot_hidden_support() {
		let dir = std::env::temp_dir().join(format!(
			"yazi-test-hidden-{}",
			std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
		));
		fs::create_dir_all(&dir).unwrap();

		fs::write(dir.join("visible.txt"), "hello").unwrap();
		fs::write(dir.join("hidden_by_file.txt"), "secret").unwrap();
		fs::write(dir.join(".dotfile"), "dot").unwrap();
		fs::write(dir.join(".hidden"), "hidden_by_file.txt\n# comment\nnonexistent.txt\n").unwrap();

		let url = UrlBuf::from(&dir);
		let files = Entries::from_dir_bulk(&url).await.unwrap();

		let visible = files.iter().find(|f| f.name().unwrap() == "visible.txt").unwrap();
		let dotfile = files.iter().find(|f| f.name().unwrap() == ".dotfile").unwrap();
		let hidden_file = files.iter().find(|f| f.name().unwrap() == "hidden_by_file.txt").unwrap();
		let dot_hidden = files.iter().find(|f| f.name().unwrap() == ".hidden").unwrap();

		assert!(!visible.is_hidden(), "visible.txt should not be hidden");
		assert!(dotfile.is_hidden(), ".dotfile should be hidden");
		assert!(hidden_file.is_hidden(), "hidden_by_file.txt should be hidden by .hidden");
		assert!(dot_hidden.is_hidden(), ".hidden itself should be hidden");

		let mut rx = Entries::from_dir(&url).await.unwrap();
		let mut streamed = Vec::new();
		while let Some(file) = rx.recv().await {
			streamed.push(file);
		}
		let hidden_streamed =
			streamed.iter().find(|f| f.name().unwrap() == "hidden_by_file.txt").unwrap();
		assert!(hidden_streamed.is_hidden(), "streamed hidden_by_file.txt should be hidden");

		_ = fs::remove_dir_all(&dir);
	}
}
