use std::{path::Path, process::Stdio};

use anyhow::{bail, Result};
use tokio::process::Command;
use yazi_shared::fs::Url;

pub struct FzfOpt {
	pub cwd: Url,
}

pub async fn fzf(opt: FzfOpt) -> Result<Url> {
	let child =
		Command::new("fzf").current_dir(&opt.cwd).kill_on_drop(true).stdout(Stdio::piped()).spawn()?;

	let output = child.wait_with_output().await?;
	let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();

	if selected.is_empty() {
		bail!("No match")
	}
	return Ok(Url::from(Path::new(&opt.cwd).join(selected)));
}
