use std::{ffi::OsString, process::Stdio};

use anyhow::Result;
use tokio::process::{Child, Command};

pub struct ShellOpt {
	pub cmd:   OsString,
	pub args:  Vec<OsString>,
	pub piped: bool,
}

pub fn shell(opt: ShellOpt) -> Result<Child> {
	#[cfg(not(target_os = "windows"))]
	{
		Ok(
			Command::new("sh")
				.arg("-c")
				.arg(opt.cmd)
				.arg("") // $0 is the command name
				.args(opt.args)
				.stdin(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.stdout(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.stderr(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.kill_on_drop(true)
				.spawn()?,
		)
	}

	#[cfg(target_os = "windows")]
	{
		Ok(
			Command::new("cmd")
				.arg("/C")
				.arg(opt.cmd)
				.args(opt.args)
				.stdin(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.stdout(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.stderr(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.kill_on_drop(true)
				.spawn()?,
		)
	}
}
