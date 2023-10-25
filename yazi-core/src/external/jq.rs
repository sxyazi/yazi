use std::{path::Path, process::Stdio};

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, AsyncReadExt, BufReader}, process::Command, select};
use yazi_config::PREVIEW;
use yazi_shared::PeekError;

pub async fn jq(path: &Path, skip: usize, limit: usize) -> Result<String, PeekError> {
	let mut child = Command::new("jq")
		.args(["-C", "--indent", "-1", "."])
		.arg(path)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.kill_on_drop(true)
		.spawn()?;

	let mut i = 0;
	let mut it = BufReader::new(child.stdout.take().unwrap()).lines();
	let mut lines = String::new();
	while let Ok(Some(line)) = it.next_line().await {
		i += 1;
		if i > skip + limit {
			break;
		}
		if i > skip {
			lines.push_str(&line);
			lines.push('\n');
		}
	}

	child.start_kill().ok();
	if lines.is_empty() {
		let mut stderr = child.stderr.take().unwrap();
		select! {
			Ok(_) = stderr.read_u8() => {
				return Err("parse error".into());
			}
		}
	}

	if skip > 0 && i < skip + limit {
		Err(PeekError::Exceed(i.saturating_sub(limit)))
	} else {
		Ok(lines.replace('\t', &" ".repeat(PREVIEW.tab_size as usize)))
	}
}
