use std::{ffi::OsString, process::Stdio};

use anyhow::Result;
use tokio::process::{Child, Command};
use yazi_fs::Cwd;
use yazi_macro::impl_data_any;
use yazi_shared::url::{AsUrl, UrlBuf};

#[derive(Clone, Debug)]
pub struct ShellOpt {
	pub cwd:    UrlBuf,
	pub cmd:    OsString,
	pub block:  bool,
	pub orphan: bool,
}

impl_data_any!(ShellOpt);

impl ShellOpt {
	#[inline]
	fn stdio(&self) -> Stdio {
		if self.block {
			Stdio::inherit()
		} else if self.orphan {
			Stdio::null()
		} else {
			Stdio::piped()
		}
	}
}

pub(crate) async fn shell(opt: ShellOpt) -> Result<Child> {
	tokio::task::spawn_blocking(move || {
		let cwd = Cwd::ensure(opt.cwd.as_url());

		#[cfg(unix)]
		return Ok(unsafe {
			Command::new("sh")
				.stdin(opt.stdio())
				.stdout(opt.stdio())
				.stderr(opt.stdio())
				.arg("-c")
				.arg(opt.cmd)
				.current_dir(cwd)
				.kill_on_drop(!opt.orphan)
				.pre_exec(move || {
					if !opt.block && libc::setsid() < 0 {
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
				.env("=", r#""^\n\n""#)
				.raw_arg(r#"/Q /S /D /V:OFF /E:ON /C ""#)
				.raw_arg(opt.cmd)
				.raw_arg(r#"""#)
				.current_dir(cwd)
				.kill_on_drop(!opt.orphan)
				.spawn()?,
		);
	})
	.await?
}
