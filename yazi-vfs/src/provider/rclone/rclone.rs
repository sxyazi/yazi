use std::{io, path::Path, process::Stdio, sync::Arc};

use tokio::sync::mpsc::Receiver;
use yazi_fs::{cha::ChaMode, provider::{Capabilities, Provider}};
use yazi_shared::{loc::LocBuf, path::{AsPath, PathBufDyn}, pool::InternStr, scheme::SchemeKind, strand::AsStrand, url::{Url, UrlBuf, UrlCow}};

use super::Item;
use crate::config::{ServiceRclone, Vfs};

#[derive(Clone)]
pub struct Rclone<'a> {
	url:  Url<'a>,
	path: &'a typed_path::UnixPath,

	name:   &'static str,
	config: &'static ServiceRclone,
}

impl<'a> Provider for Rclone<'a> {
	type File = super::File;
	type Gate = super::Gate;
	type Me<'b> = Rclone<'b>;
	type ReadDir = super::ReadDir;
	type UrlCow = UrlCow<'a>;

	async fn absolute(&self) -> io::Result<Self::UrlCow> {
		Ok(if let Some(u) = super::try_absolute(self.url) { u } else { self.url.to_owned().into() })
	}

	async fn canonicalize(&self) -> io::Result<UrlBuf> { Ok(self.url.to_owned()) }

	fn capabilities(&self) -> Capabilities { Capabilities { symlink: false } }

	async fn casefold(&self) -> io::Result<UrlBuf> { Ok(self.url.to_owned()) }

	async fn copy<P>(&self, _to: P, _attrs: yazi_fs::provider::Attrs) -> io::Result<u64>
	where
		P: AsPath,
	{
		Err(read_only())
	}

	fn copy_with_progress<P, A>(&self, to: P, attrs: A) -> io::Result<Receiver<io::Result<u64>>>
	where
		P: AsPath,
		A: Into<yazi_fs::provider::Attrs>,
	{
		let to = UrlBuf::Rclone {
			loc:    LocBuf::<typed_path::UnixPathBuf>::saturated(
				to.as_path().to_unix_owned()?,
				SchemeKind::Rclone,
			),
			domain: self.name.intern(),
		};
		let from = self.url.to_owned();

		Ok(crate::provider::copy_with_progress_impl(from, to, attrs.into()))
	}

	async fn create_dir(&self) -> io::Result<()> { Err(read_only()) }

	async fn hard_link<P>(&self, _to: P) -> io::Result<()>
	where
		P: AsPath,
	{
		Err(io::Error::new(io::ErrorKind::Unsupported, "Hard links not supported"))
	}

	async fn metadata(&self) -> io::Result<yazi_fs::cha::Cha> {
		stat(self.config, &target(self.config, self.path)?).await?.cha()
	}

	async fn new<'b>(url: Url<'b>) -> io::Result<Self::Me<'b>> {
		match url {
			Url::Rclone { loc, domain } => {
				let (name, config) = Vfs::service::<&ServiceRclone>(domain).await?;
				Ok(Self::Me { url, path: loc.as_inner(), name, config })
			}
			_ => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Not an rclone URL: {url:?}"))),
		}
	}

	async fn read_dir(self) -> io::Result<Self::ReadDir> {
		let mut cmd = command(self.config);
		cmd.arg("lsjson").arg(target(self.config, self.path)?);

		let items: Vec<Item> = serde_json::from_slice(&output(cmd).await?)
			.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

		Ok(Self::ReadDir { dir: Arc::new(self.url.to_owned()), entries: items.into() })
	}

	async fn read_link(&self) -> io::Result<PathBufDyn> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Symlinks not supported"))
	}

	async fn remove_dir(&self) -> io::Result<()> { Err(read_only()) }

	async fn remove_file(&self) -> io::Result<()> { Err(read_only()) }

	async fn rename<P>(&self, _to: P) -> io::Result<()>
	where
		P: AsPath,
	{
		Err(read_only())
	}

	async fn set_mode(&self, _mode: ChaMode) -> io::Result<()> { Err(read_only()) }

	async fn symlink<S, F>(&self, _original: S, _is_dir: F) -> io::Result<()>
	where
		S: AsStrand,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		Err(io::Error::new(io::ErrorKind::Unsupported, "Symlinks not supported"))
	}

	async fn symlink_metadata(&self) -> io::Result<yazi_fs::cha::Cha> { self.metadata().await }

	async fn trash(&self) -> io::Result<()> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Trash not supported"))
	}

	#[inline]
	fn url(&self) -> Url<'_> { self.url }
}

// --- Helpers
#[inline]
pub(super) fn read_only() -> io::Error {
	io::Error::new(io::ErrorKind::Unsupported, "The rclone provider is currently read-only")
}

pub(super) fn command(config: &ServiceRclone) -> tokio::process::Command {
	let binary =
		if config.binary.as_os_str().is_empty() { Path::new("rclone") } else { &config.binary };

	let mut cmd = tokio::process::Command::new(binary);
	if !config.config_file.as_os_str().is_empty() {
		cmd.arg("--config").arg(&config.config_file);
	}

	cmd.args(&config.flags);
	cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
	cmd.kill_on_drop(true);
	cmd
}

pub(super) fn target(config: &ServiceRclone, path: &typed_path::UnixPath) -> io::Result<String> {
	// Keep the leading slash: object stores treat `remote:/bucket/key` and
	// `remote:bucket/key` identically, while rclone's filesystem-like backends
	// (local, sftp, …) need the absolute path preserved (`remote:/abs/path`).
	let s = str::from_utf8(path.as_bytes())
		.map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Non-UTF-8 path"))?;

	Ok(format!("{}:{s}", config.remote))
}

pub(super) async fn output(mut cmd: tokio::process::Command) -> io::Result<Vec<u8>> {
	let output = cmd.output().await?;
	if output.status.success() {
		return Ok(output.stdout);
	}

	// https://rclone.org/docs/#exit-code — 3: directory not found, 4: file not found
	let kind = match output.status.code() {
		Some(3 | 4) => io::ErrorKind::NotFound,
		_ => io::ErrorKind::Other,
	};
	let stderr = String::from_utf8_lossy(&output.stderr);
	Err(io::Error::new(kind, format!("rclone failed: {}", stderr.trim())))
}

pub(super) async fn stat(config: &ServiceRclone, target: &str) -> io::Result<Item> {
	let mut cmd = command(config);
	cmd.arg("lsjson").arg("--stat").arg(target);

	serde_json::from_slice(&output(cmd).await?)
		.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
