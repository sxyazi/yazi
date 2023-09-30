use std::{ffi::OsString, process::Stdio};

use anyhow::Result;
use tokio::process::{Child, Command};

pub struct ShellOpt {
	pub cmd:    OsString,
	pub args:   Vec<OsString>,
	pub piped:  bool,
	pub orphan: bool,
}

impl ShellOpt {
	pub fn with_piped(mut self) -> Self {
		self.piped = true;
		self
	}

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

pub fn shell(opt: ShellOpt) -> Result<Child> {
	#[cfg(unix)]
	return Ok(
		Command::new("sh")
			.arg("-c")
			.stdin(opt.stdio())
			.stdout(opt.stdio())
			.stderr(opt.stdio())
			.arg(opt.cmd)
			.arg("") // $0 is the command name
			.args(opt.args)
			.kill_on_drop(!opt.orphan)
			.spawn()?,
	);

	#[cfg(windows)]
	return Ok(
		Command::new("cmd")
			.stdin(opt.stdio())
			.stdout(opt.stdio())
			.stderr(opt.stdio())
			.arg("/C")
			.arg(opt.cmd)
			.args(opt.args)
			.kill_on_drop(true)
			.spawn()?,
	);
}
