use std::{borrow::Cow, ffi::OsString, process::Stdio};

use anyhow::{Result, bail};
use tokio::process::{Child, Command};
use yazi_fs::FsUrl;
use yazi_shared::url::{UrlBuf, UrlCow, UrlLike};

pub(crate) struct ShellOpt {
	pub(crate) cwd:    UrlBuf,
	pub(crate) cmd:    OsString,
	pub(crate) args:   Vec<UrlCow<'static>>,
	pub(crate) piped:  bool,
	pub(crate) orphan: bool,
}

impl ShellOpt {
	#[inline]
	fn stdio(&self) -> Stdio {
		if self.orphan {
			Stdio::null()
		} else if self.piped {
			Stdio::piped()
		} else {
			Stdio::inherit()
		}
	}
}

pub(crate) async fn shell(opt: ShellOpt) -> Result<Child> {
	tokio::task::spawn_blocking(move || {
		let cwd: Cow<_> = if let Some(path) = opt.cwd.as_path() {
			path.into()
		} else if let Some(cache) = opt.cwd.cache() {
			std::fs::create_dir_all(&cache).ok();
			cache.into()
		} else {
			bail!("failed to determine a working directory");
		};

		#[cfg(unix)]
		return Ok(unsafe {
			use yazi_shared::url::AsUrl;

			Command::new("sh")
				.stdin(opt.stdio())
				.stdout(opt.stdio())
				.stderr(opt.stdio())
				.arg("-c")
				.arg(opt.cmd)
				.arg("--")
				// TODO: remove
				.args(opt.args.iter().skip(1).map(|u| u.as_url().unified_path_str()))
				.current_dir(cwd)
				.kill_on_drop(!opt.orphan)
				.pre_exec(move || {
					if (opt.piped || opt.orphan) && libc::setsid() < 0 {
						return Err(std::io::Error::last_os_error());
					}
					Ok(())
				})
				.spawn()?
		});

		#[cfg(windows)]
		return Ok(
			Command::new("cmd.exe")
				.stdin(opt.stdio())
				.stdout(opt.stdio())
				.stderr(opt.stdio())
				.raw_arg("/C")
				.raw_arg(opt.cmd)
				.current_dir(cwd)
				.kill_on_drop(!opt.orphan)
				.spawn()?,
		);
	})
	.await?
}
