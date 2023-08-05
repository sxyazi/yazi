use std::{path::Path, process::Stdio};

use anyhow::{bail, Result};
use config::PREVIEW;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command};

pub async fn jq(path: &Path, mut lines: usize) -> Result<String> {
	let mut child = Command::new("jq")
		.args(["-C", "--indent", &PREVIEW.tab_size.to_string(), "."])
		.arg(path)
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.kill_on_drop(true)
		.spawn()?;

	let mut it = BufReader::new(child.stdout.take().unwrap()).lines();
	let mut output = String::new();
	while let Ok(Some(line)) = it.next_line().await {
		if lines < 1 {
			break;
		}

		output.push_str(&line);
		output.push('\n');
		lines -= 1;
	}

	if output.is_empty() {
		bail!("failed to get head of jq");
	}
	Ok(output)
}
