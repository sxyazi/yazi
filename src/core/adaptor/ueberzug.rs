use std::{path::PathBuf, process::Stdio};

use anyhow::Result;
use ratatui::prelude::Rect;
use tokio::{io::AsyncWriteExt, process::{Child, Command}, sync::mpsc::{self, UnboundedSender}};

use crate::config::PREVIEW;

pub(super) struct Ueberzug;

impl Ueberzug {
	pub(super) fn start() -> Result<UnboundedSender<Option<(PathBuf, Rect)>>> {
		let mut child = Self::create_demon().ok();
		let (tx, mut rx) = mpsc::unbounded_channel();

		tokio::spawn(async move {
			while let Some(cmd) = rx.recv().await {
				let exit = child.as_mut().and_then(|c| c.try_wait().ok());
				if exit != Some(None) {
					child = None;
				}
				if child.is_none() {
					child = Self::create_demon().ok();
				}
				if let Some(c) = &mut child {
					Self::send_command(c, cmd).await.ok();
				}
			}
		});

		Ok(tx)
	}

	fn create_demon() -> Result<Child> {
		Ok(
			Command::new("ueberzug")
				.args(["layer", "-so", &PREVIEW.adaptor.to_string()])
				.kill_on_drop(true)
				.stdin(Stdio::piped())
				.stderr(Stdio::null())
				.spawn()?,
		)
	}

	async fn send_command(child: &mut Child, cmd: Option<(PathBuf, Rect)>) -> Result<()> {
		let stdin = child.stdin.as_mut().unwrap();
		if let Some((path, rect)) = cmd {
			let s = format!(
				r#"{{"action":"add","identifier":"yazi","x":{},"y":{},"max_width":{},"max_height":{},"path":"{}"}}{}"#,
				rect.x,
				rect.y,
				rect.width,
				rect.height,
				path.to_string_lossy(),
				"\n"
			);
			stdin.write_all(s.as_bytes()).await?;
		} else {
			stdin
				.write_all(format!(r#"{{"action":"remove","identifier":"yazi"}}{}"#, "\n").as_bytes())
				.await?;
		}
		Ok(())
	}
}
