use std::{path::Path, process::Stdio};

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command};
use yazi_config::PREVIEW;
use yazi_shared::PeekError;

pub async fn jq(path: &Path, skip: usize, limit: usize) -> Result<String, PeekError> {
	let mut child = Command::new("jq")
		.args(["-C", "--indent", &PREVIEW.tab_size.to_string(), "."])
		.arg(path)
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.kill_on_drop(true)
		.spawn()?;

	let mut i = 0;
	let mut it = BufReader::new(child.stdout.take().unwrap()).lines();
	let mut lines = String::new();
	while let Ok(Some(line)) = it.next_line().await {
		i += 1;
		if i > skip + limit {
			break;
		} else if i <= skip {
			continue;
		}

		lines.push_str(&line);
		lines.push('\n');
	}

	child.start_kill().ok();
	if skip > 0 && i < skip + limit {
		Err(PeekError::Exceed(i.saturating_sub(limit)))
	} else {
		Ok(lines)
	}
}
