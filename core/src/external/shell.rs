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

	#[cfg(target_os = "windows")]
	{
		let args: Vec<String> = opt.args.iter().map(|s| s.to_string_lossy().to_string()).collect();
		let cmd = cmdexpand::Expander::new(&opt.cmd.to_string_lossy().to_string())
			.disable_context(true)
			.add_args(&args)
			.expand()?;
		Ok(
			Command::new("cmd")
				.arg("/C")
				.arg(cmd)
				.stdin(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.stdout(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.stderr(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.kill_on_drop(true)
				.spawn()?,
		)
	}
}
